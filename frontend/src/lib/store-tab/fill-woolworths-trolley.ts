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
 * 2. Read the current Woolworths trolley, so each product is *added on top*
 *    of what is already there — the trolley call sets a quantity.
 * 3. Set each product's new quantity through `/api/v3/ui/trolley/update`,
 *    one product per call so one refusal cannot hide the others.
 * 4. Report every product's outcome to our backend, and tell the person.
 *
 * `fillWoolworthsTrolley` must stay **self-contained** — no imports, no
 * outside variables — because the bookmarklet carries its source text
 * (`Function.prototype.toString`). Endpoints are from the live site,
 * checked 2026-10-02 (docs/FEAT_WOOLWORTHS_ACCESS.md).
 */

/** What the bookmarklet is built with. */
export interface FillTrolleyConfig {
	/** Our backend, e.g. `https://grocery.example.com` (no trailing slash). */
	apiBaseUrl: string;
	/** `STORE_TAB_SECRET`, from `GET /api/trolley-handoffs/store-tab-secret`. */
	secret: string;
}

/** What one run did, also shown to the person. */
export interface FillTrolleyResult {
	/** `null` when nothing was waiting. */
	handoffId: string | null;
	added: string[];
	failed: { productId: string; problem: string }[];
	/** Added, but with a warning from Woolworths worth showing. */
	notes?: { productId: string; problem: string }[];
	message: string;
}

/** Fills the Woolworths trolley from the newest waiting handoff. */
export async function fillWoolworthsTrolley(
	config: FillTrolleyConfig,
): Promise<FillTrolleyResult> {
	const WOOLWORTHS_JSON = {
		accept: "application/json, text/plain, */*",
		"content-type": "application/json",
	};
	const sendToBackend = (path: string, body: unknown) =>
		fetch(`${config.apiBaseUrl}${path}`, {
			method: "POST",
			headers: { "content-type": "text/plain;charset=UTF-8" },
			body: JSON.stringify({ ...(body as object), secret: config.secret }),
		});

	if (!/(^|\.)woolworths\.com\.au$/.test(location.hostname)) {
		return {
			handoffId: null,
			added: [],
			failed: [],
			message: "Open woolworths.com.au first, then press the bookmark.",
		};
	}

	const claimed = await sendToBackend("/api/store-tab/trolley-handoffs/claim", {
		store: "woolworths",
	});
	if (claimed.status === 204) {
		return {
			handoffId: null,
			added: [],
			failed: [],
			message:
				"Nothing to add. Press “Send to Woolworths” in the grocery app first.",
		};
	}
	if (!claimed.ok) {
		throw new Error(`The grocery app refused (HTTP ${claimed.status}).`);
	}
	const handoff = (await claimed.json()) as {
		handoff_id: string;
		lines: { product_id: string; name: string; quantity: number }[];
	};

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
	const notes: { productId: string; problem: string }[] = [];
	for (const line of handoff.lines) {
		const stockcode = Number(line.product_id);
		if (!Number.isInteger(stockcode)) {
			failed.push({
				productId: line.product_id,
				problem: "Not a Woolworths stockcode",
			});
			continue;
		}
		const quantity = (already.get(line.product_id) ?? 0) + line.quantity;
		try {
			const response = await fetch("/api/v3/ui/trolley/update", {
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
			// Woolworths answers `IsAvailable: false` (and a $0 price) for
			// products that *are* now in the trolley when no delivery time is
			// picked yet, so success is the trolley quantity, not that flag.
			if (!updated || Number(updated.QuantityInTrolley) < quantity) {
				failed.push({
					productId: line.product_id,
					problem: answer?.Message || "Woolworths did not add this product",
				});
			} else {
				added.push(line.product_id);
				if (updated.IsAvailable === false) {
					notes.push({
						productId: line.product_id,
						problem:
							"In your trolley, but Woolworths says it is not available yet — pick a delivery time to check",
					});
				}
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
			lines: [
				...added.map((product_id) => ({
					product_id,
					outcome: "added",
					problem: notes.find((n) => n.productId === product_id)?.problem,
				})),
				...failed.map((f) => ({
					product_id: f.productId,
					outcome: "failed",
					problem: f.problem.slice(0, 300),
				})),
			],
		},
	);

	const message =
		failed.length === 0
			? `Added ${added.length} products to your trolley. Check it, then pay on Woolworths.`
			: `Added ${added.length}. ${failed.length} could not be added — the grocery app shows which.`;
	return { handoffId: handoff.handoff_id, added, failed, notes, message };
}
