/**
 * The Coles website and our backend as the Coles fill script sees them,
 * shared by the Coles fill and bookmarklet tests.
 */
import { vi } from "vitest";

export const config = { apiBaseUrl: "https://grocery.test", secret: "s3cret" };
const TROLLEY = "/api/bff/trolley/store/0584";

/**
 * A fake network: our backend plus the Coles website's trolley call,
 * behaving as the real one did on 2026-10-06. Reading the trolley needs
 * `x-api-version: 3` (404 without it) and lists products under `items`. A
 * PATCH without `orderItemId` **adds** the quantity sent. A product Coles
 * refuses (`refused`) comes back in `failedItems` with HTTP 200; one it
 * silently ignores (`ignored`) is reported as added but does not appear.
 */
export function fakeNetwork({
	claim = {
		status: 200,
		body: {
			handoff_id: "h1",
			store: "coles",
			lines: [
				{ product_id: "123011", name: "Coke 1.25L", quantity: 2 },
				{ product_id: "409499", name: "Bananas", quantity: 1 },
			],
		},
	},
	inTrolley = { "123011": 1 } as Record<string, number>,
	trolleyStatus = 200,
	refused = [] as string[],
	ignored = [] as string[],
	status400 = [] as string[],
	slot = null as Record<string, string> | null,
}: {
	claim?: { status: number; body?: unknown };
	inTrolley?: Record<string, number>;
	trolleyStatus?: number;
	refused?: string[];
	ignored?: string[];
	status400?: string[];
	slot?: Record<string, string> | null;
} = {}) {
	const trolley = new Map(Object.entries(inTrolley));
	const calls: {
		url: string;
		method: string;
		body: unknown;
		headers: Record<string, string>;
	}[] = [];
	const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
		const body = init?.body ? JSON.parse(String(init.body)) : undefined;
		const method = init?.method ?? "GET";
		const headers = (init?.headers ?? {}) as Record<string, string>;
		calls.push({ url, method, body, headers });
		const json = (status: number, value: unknown) =>
			new Response(value === undefined ? null : JSON.stringify(value), {
				status,
			});
		if (url.endsWith("/claim")) return json(claim.status, claim.body);
		if (url.endsWith("/report")) return json(200, {});
		if (url === `${TROLLEY}?sortBy=recentlyAdded` && method === "GET") {
			if (trolleyStatus !== 200) return json(trolleyStatus, {});
			if (headers["x-api-version"] !== "3") return json(404, {});
			return json(200, {
				items: [...trolley].map(([productId, quantity]) => ({
					orderItemId: `o-${productId}`,
					productId: Number(productId),
					quantity,
				})),
				orderAttributes: { slot },
			});
		}
		if (url === TROLLEY && method === "PATCH") {
			const { productId, quantity } = body.items[0].actions[0];
			if (status400.includes(productId)) return json(400, { message: "No" });
			if (refused.includes(productId)) {
				return json(200, {
					results: [
						{
							actionedItems: [],
							failedItems: [
								{
									productId,
									quantity,
									action: "ADD",
									error: {
										errorCode: "unavailable",
										message: "Product is unavailable",
									},
								},
							],
						},
					],
				});
			}
			if (!ignored.includes(productId)) {
				trolley.set(productId, (trolley.get(productId) ?? 0) + quantity);
			}
			return json(200, {
				results: [
					{
						actionedItems: [{ productId, quantity, action: "ADD" }],
						failedItems: [],
					},
				],
			});
		}
		return json(404, null);
	});
	vi.stubGlobal("fetch", fetchMock);
	return { calls, trolley };
}

/** The Coles page as a logged-in household member sees it. */
export function colesPage({
	apiKey = "bff-key" as string | null,
	storeId = "0584" as string | null,
} = {}) {
	vi.stubGlobal("location", { hostname: "www.coles.com.au" });
	vi.stubGlobal(
		"__RUNTIME_CONFIG__",
		apiKey ? { BFF_API_SUBSCRIPTION_KEY: apiKey } : undefined,
	);
	localStorage.clear();
	if (storeId) {
		localStorage.setItem(
			"shoppingMethod",
			JSON.stringify({ currentFulfilmentStoreId: storeId }),
		);
	}
	vi.spyOn(document, "cookie", "get").mockReturnValue(
		"sessionId=sess-1; visitorId=visit-1",
	);
}

export const reportOf = (calls: { url: string; body: unknown }[]) =>
	calls.find((c) => c.url.endsWith("/report"))?.body as {
		secret: string;
		delivery: { outcome: string; problem: string };
		lines: { product_id: string; outcome: string; problem?: string }[];
	};
