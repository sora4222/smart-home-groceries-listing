/**
 * The script the "Fill Woolworths trolley" bookmarklet runs **on
 * woolworths.com.au**, inside the household member's own logged-in tab.
 *
 * Why it runs there: Woolworths login uses passkeys and MFA, and its website
 * sits behind Akamai bot protection. A tab the person is already logged in
 * to has the real session, the real browser fingerprint and their home IP,
 * so the website's own trolley call simply works. Nothing here logs in,
 * reads cookies, or touches payment.
 *
 * Steps:
 * 1. Claim the newest waiting handoff from our backend (secret in the body,
 *    sent as `text/plain` so the browser sends no CORS preflight).
 * 2. Reserve the delivery time asked for (default: the next day) with
 *    `helpers.reserveDeliveryWindow`, so availability is for a real delivery.
 *    The household can change it on Woolworths later.
 * 3. Read the current Woolworths trolley, so each product is *added on top*
 *    of what is already there — the trolley call sets a quantity.
 * 4. Set each product's new quantity through `/api/v3/ui/trolley/update`,
 *    one product per call so one refusal cannot hide the others. A product
 *    Woolworths says is unavailable at the household's store is put back to
 *    its old quantity and reported as not added.
 * 5. Report every product's outcome to our backend, and tell the person.
 * 6. When anything was added, offer the checkout page (`/shop/checkout`).
 *    The person reviews the order and pays there; this never pays.
 *
 * `fillWoolworthsTrolley` must stay **self-contained** — no imports, no
 * outside variables — because the bookmarklet carries its source text
 * (`Function.prototype.toString`). The helpers it needs arrive as its
 * second argument, each self-contained too (`bookmarklet.ts` wires them). Endpoints are from the live site,
 * checked 2026-10-02 (docs/FEAT_WOOLWORTHS_ACCESS.md).
 * Its place in the web app is `WOOLWORTHS_TAB` (`store-tabs.ts`).
 */

import type {
	ChosenWindow,
	DeliveryWanted,
	WoolworthsDay,
} from "#/lib/store-tab/choose-woolworths-window";
import type { DeliveryReport } from "#/lib/store-tab/reserve-woolworths-delivery-window";
import type {
	FillTrolleyConfig,
	FillTrolleyResult,
} from "#/lib/store-tab/store-tab";

/** The self-contained helpers the script is handed. */
export interface FillTrolleyHelpers {
	chooseWindow: (
		days: WoolworthsDay[],
		wanted: DeliveryWanted,
		storeToday: string,
	) => ChosenWindow | null;
	reserveDeliveryWindow: (
		wanted: DeliveryWanted,
		chooseWindow: FillTrolleyHelpers["chooseWindow"],
	) => Promise<DeliveryReport>;
}

/** Fills the Woolworths trolley from the newest waiting handoff. */
export async function fillWoolworthsTrolley(
	config: FillTrolleyConfig,
	helpers: FillTrolleyHelpers,
): Promise<FillTrolleyResult> {
	const WOOLWORTHS_JSON = {
		accept: "application/json, text/plain, */*",
		"content-type": "application/json",
	};
	const nothing = (message: string): FillTrolleyResult => ({
		handoffId: null,
		added: [],
		failed: [],
		delivery: null,
		message,
		checkout: null,
	});
	const sendToBackend = (path: string, body: unknown) =>
		fetch(`${config.apiBaseUrl}${path}`, {
			method: "POST",
			headers: { "content-type": "text/plain;charset=UTF-8" },
			body: JSON.stringify({ ...(body as object), secret: config.secret }),
		});
	const setQuantity = (stockcode: number, quantity: number) =>
		fetch("/api/v3/ui/trolley/update", {
			method: "POST",
			credentials: "include",
			headers: WOOLWORTHS_JSON,
			body: JSON.stringify({
				items: [
					{
						stockcode,
						quantity,
						source: "ProductDetail",
						diagnostics: "0",
						searchTerm: null,
						evaluateRewardPoints: false,
						offerId: null,
						profileId: null,
						priceLevel: null,
					},
				],
			}),
		});

	if (!/(^|\.)woolworths\.com\.au$/.test(location.hostname)) {
		return nothing("Open woolworths.com.au first, then press the bookmark.");
	}

	const claimed = await sendToBackend("/api/store-tab/trolley-handoffs/claim", {
		store: "woolworths",
	});
	if (claimed.status === 204) {
		return nothing(
			"Nothing to add. Press “Send to Woolworths” in the grocery app first.",
		);
	}
	if (!claimed.ok) {
		throw new Error(`The grocery app refused (HTTP ${claimed.status}).`);
	}
	const handoff = (await claimed.json()) as {
		handoff_id: string;
		delivery?: DeliveryWanted;
		lines: { product_id: string; name: string; quantity: number }[];
	};

	const delivery = await helpers.reserveDeliveryWindow(
		handoff.delivery ?? { date: null, time_of_day: "any" },
		helpers.chooseWindow,
	);

	const trolley = await fetch("/api/v3/ui/trolley", {
		credentials: "include",
		headers: WOOLWORTHS_JSON,
		cache: "no-store",
	}).then((r) => (r.ok ? r.json() : { Products: [] }));
	const already = new Map<string, number>();
	for (const product of trolley.Products ?? []) {
		already.set(
			String(product.Stockcode),
			Number(product.QuantityInTrolley) || 0,
		);
	}

	const added: string[] = [];
	const failed: { productId: string; problem: string }[] = [];
	for (const line of handoff.lines) {
		const stockcode = Number(line.product_id);
		if (!Number.isInteger(stockcode)) {
			failed.push({
				productId: line.product_id,
				problem: "Not a Woolworths stockcode",
			});
			continue;
		}
		const before = already.get(line.product_id) ?? 0;
		const quantity = before + line.quantity;
		try {
			const response = await setQuantity(stockcode, quantity);
			if (!response.ok) {
				failed.push({
					productId: line.product_id,
					problem: `Woolworths answered HTTP ${response.status}`,
				});
				continue;
			}
			const answer = await response.json();
			const updated = answer?.UpdatedItems?.find(
				(item: { Stockcode: number }) => item.Stockcode === stockcode,
			);
			if (!updated || Number(updated.QuantityInTrolley) < quantity) {
				failed.push({
					productId: line.product_id,
					problem: answer?.Message || "Woolworths did not add this product",
				});
			} else if (updated.IsAvailable === false) {
				// Unavailable at the household's store: it would not be
				// delivered, so take it back out rather than leave it there.
				await setQuantity(stockcode, before);
				failed.push({
					productId: line.product_id,
					problem: "Not available at your Woolworths store right now",
				});
			} else {
				added.push(line.product_id);
			}
		} catch (error) {
			failed.push({
				productId: line.product_id,
				problem: String(error).slice(0, 300),
			});
		}
	}

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

	const when =
		delivery.outcome === "failed"
			? `No delivery time: ${delivery.problem}`
			: `Delivery: ${delivery.window_start?.slice(0, 10)}, ${delivery.window_label}. Change it on Woolworths any time.`;
	const products =
		failed.length === 0
			? `Added ${added.length} products to your trolley.`
			: `Added ${added.length}. ${failed.length} could not be added — the grocery app shows which.`;
	return {
		handoffId: handoff.handoff_id,
		added,
		failed,
		delivery,
		message: `${products}\n${when}`,
		checkout:
			added.length > 0
				? {
						path: "/shop/checkout",
						question:
							"Go to checkout now? You check the order and pay on Woolworths. Nothing is bought until you do.",
					}
				: null,
	};
}
