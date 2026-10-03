/**
 * The script the "Fill Coles trolley" bookmarklet runs **on coles.com.au**,
 * inside the household member's own logged-in tab.
 *
 * Why it runs there: the server cannot log in to Coles, and the website sits
 * behind bot protection. A tab the person is already logged in to has the
 * real session, so the website's own trolley call simply works. Nothing here
 * logs in, stores a cookie, or touches payment. The cookies it reads
 * (`sessionId`, `visitorId`, `dsch-ccpuserid`) are only copied into the
 * headers the Coles website itself sends with every trolley call.
 *
 * Steps:
 * 1. Check the page is ready: the website's API key
 *    (`window.__RUNTIME_CONFIG__.BFF_API_SUBSCRIPTION_KEY`) and the
 *    household's chosen Coles store. Then read the trolley, which also proves
 *    the person is logged in. Only then claim the handoff, so a page that is
 *    not ready leaves it waiting for the next press.
 * 2. Claim the newest waiting Coles handoff from our backend.
 * 3. Set each product's new quantity (what is there **plus** the list's
 *    quantity) through `PATCH /api/bff/trolley/store/{storeId}`, one product
 *    per call so one refusal cannot hide the others.
 * 4. Read the trolley again: a product counts as added only when its
 *    quantity really went up.
 * 5. Report every product to our backend. The delivery time is reported as
 *    not chosen: the Coles delivery-time calls are not known yet, so the
 *    person picks one on Coles.
 *
 * Endpoints: `GET`/`PATCH /api/bff/trolley/store/{storeId}`, taken from an
 * open-source bookmarklet that uses them (coles-vs-woolies,
 * `static/cart-bookmarklet.js`). **Not yet checked live by this project** —
 * the build workspace cannot reach coles.com.au.
 *
 * `fillColesTrolley` must stay **self-contained** — no imports at run time,
 * no outside variables — because the bookmarklet carries its source text.
 * Its place in the web app is `COLES_TAB` (`store-tabs.ts`).
 */
import type {
	FillTrolleyConfig,
	FillTrolleyResult,
} from "#/lib/store-tab/store-tab";

/** Fills the Coles trolley from the newest waiting Coles handoff. */
export async function fillColesTrolley(
	config: FillTrolleyConfig,
): Promise<FillTrolleyResult> {
	const nothing = (message: string): FillTrolleyResult => ({
		handoffId: null,
		added: [],
		failed: [],
		delivery: null,
		message,
		checkout: null,
	});
	const cookie = (name: string): string => {
		const found = document.cookie
			.split("; ")
			.find((part) => part.startsWith(`${name}=`));
		return found ? decodeURIComponent(found.slice(name.length + 1)) : "";
	};
	const sendToBackend = (path: string, body: unknown) =>
		fetch(`${config.apiBaseUrl}${path}`, {
			method: "POST",
			headers: { "content-type": "text/plain;charset=UTF-8" },
			body: JSON.stringify({ ...(body as object), secret: config.secret }),
		});

	if (!/(^|\.)coles\.com\.au$/.test(location.hostname)) {
		return nothing("Open coles.com.au first, then press the bookmark.");
	}

	// 1. Is the page ready? (Nothing is claimed yet.)
	const runtime = (window as unknown as Record<string, unknown>)
		.__RUNTIME_CONFIG__ as { BFF_API_SUBSCRIPTION_KEY?: string } | undefined;
	const apiKey = runtime?.BFF_API_SUBSCRIPTION_KEY;
	if (!apiKey) {
		return nothing(
			"Coles is still loading. Reload the page, then press again.",
		);
	}
	let storeId = "";
	try {
		const method = JSON.parse(localStorage.getItem("shoppingMethod") ?? "{}");
		storeId = String(method?.currentFulfilmentStoreId ?? "");
	} catch {
		storeId = "";
	}
	storeId = storeId || cookie("fulfillmentStoreId");
	if (!/^\d+$/.test(storeId)) {
		return nothing(
			"Choose your delivery address on Coles first, then press again.",
		);
	}
	const trolleyUrl = `/api/bff/trolley/store/${storeId}`;
	const colesCall = (method: "GET" | "PATCH", body?: unknown) =>
		fetch(trolleyUrl, {
			method,
			credentials: "include",
			cache: "no-store",
			headers: {
				accept: "application/json",
				"content-type": "application/json",
				"ocp-apim-subscription-key": apiKey,
				"cusp-session-id": cookie("sessionId"),
				"cusp-visitor-id": cookie("visitorId"),
				"cusp-user-id": cookie("dsch-ccpuserid"),
				"cusp-correlation-id": crypto.randomUUID(),
			},
			body: body === undefined ? undefined : JSON.stringify(body),
		});
	const readTrolley = async (): Promise<Map<string, number> | number> => {
		const response = await colesCall("GET");
		if (!response.ok) return response.status;
		const trolley = await response.json();
		const quantities = new Map<string, number>();
		for (const item of trolley?.allItems ?? []) {
			quantities.set(String(item.productId), Number(item.quantity) || 0);
		}
		return quantities;
	};

	const before = await readTrolley();
	if (before === 401 || before === 403) {
		return nothing("Log in to Coles first, then press the bookmark again.");
	}
	if (typeof before === "number") {
		throw new Error(`Coles did not show your trolley (HTTP ${before}).`);
	}

	// 2. Claim the handoff.
	const claimed = await sendToBackend("/api/store-tab/trolley-handoffs/claim", {
		store: "coles",
	});
	if (claimed.status === 204) {
		return nothing(
			"Nothing to add. Press “Send to Coles” in the grocery app first.",
		);
	}
	if (!claimed.ok) {
		throw new Error(`The grocery app refused (HTTP ${claimed.status}).`);
	}
	const handoff = (await claimed.json()) as {
		handoff_id: string;
		lines: { product_id: string; name: string; quantity: number }[];
	};

	// 3. Set each product's new quantity.
	const wanted = new Map<string, number>();
	const failed: { productId: string; problem: string }[] = [];
	for (const line of handoff.lines) {
		if (!/^\d+$/.test(line.product_id)) {
			failed.push({
				productId: line.product_id,
				problem: "Not a Coles product number",
			});
			continue;
		}
		const quantity = (before.get(line.product_id) ?? 0) + line.quantity;
		try {
			const response = await colesCall("PATCH", {
				ageGateVerified: false,
				swapBehaviour: false,
				items: [{ actions: [{ productId: line.product_id, quantity }] }],
			});
			if (response.ok) {
				wanted.set(line.product_id, quantity);
			} else {
				failed.push({
					productId: line.product_id,
					problem: `Coles answered HTTP ${response.status}`,
				});
			}
		} catch (error) {
			failed.push({
				productId: line.product_id,
				problem: String(error).slice(0, 300),
			});
		}
	}

	// 4. Check what really went in.
	const after = wanted.size > 0 ? await readTrolley() : new Map();
	const added: string[] = [];
	for (const [productId, quantity] of wanted) {
		const got = typeof after === "number" ? 0 : (after.get(productId) ?? 0);
		if (got >= quantity) {
			added.push(productId);
		} else {
			failed.push({
				productId,
				problem:
					"Coles did not add this product. It may be unavailable at your store.",
			});
		}
	}

	// 5. Report back.
	const delivery = {
		outcome: "failed" as const,
		problem: "The app cannot pick a Coles delivery time yet",
	};
	await sendToBackend(
		`/api/store-tab/trolley-handoffs/${handoff.handoff_id}/report`,
		{
			delivery,
			lines: [
				...added.map((product_id) => ({ product_id, outcome: "added" })),
				...failed.map((f) => ({
					product_id: f.productId,
					outcome: "failed",
					problem: f.problem.slice(0, 300),
				})),
			],
		},
	);

	const products =
		failed.length === 0
			? `Added ${added.length} products to your Coles trolley.`
			: `Added ${added.length}. ${failed.length} could not be added — the grocery app shows which.`;
	return {
		handoffId: handoff.handoff_id,
		added,
		failed,
		delivery,
		message: `${products}\nNow pick a delivery time on Coles, then check your trolley.`,
		// Not checked live: the build workspace cannot reach coles.com.au.
		checkout:
			added.length > 0
				? {
						path: "/checkout",
						question:
							"Go to checkout now? You pick a delivery time, check the order and pay on Coles. Nothing is bought until you do.",
					}
				: null,
	};
}
