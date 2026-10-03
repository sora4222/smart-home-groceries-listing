import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fillColesTrolley } from "#/lib/store-tab/fill-coles-trolley";
import { COLES_TAB } from "#/lib/store-tab/store-tabs";

const config = { apiBaseUrl: "https://grocery.test", secret: "s3cret" };
const TROLLEY = "/api/bff/trolley/store/0584";

/**
 * A fake network: our backend plus the Coles website's trolley call. The
 * trolley keeps quantities like the real one: a PATCH sets the quantity,
 * except for products Coles refuses (`refused`) or silently ignores
 * (`ignored`, e.g. unavailable at the household's store).
 */
function fakeNetwork({
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
}: {
	claim?: { status: number; body?: unknown };
	inTrolley?: Record<string, number>;
	trolleyStatus?: number;
	refused?: string[];
	ignored?: string[];
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
		if (url === TROLLEY && method === "GET") {
			if (trolleyStatus !== 200) return json(trolleyStatus, {});
			return json(200, {
				allItems: [...trolley].map(([productId, quantity]) => ({
					productId: Number(productId),
					quantity,
				})),
			});
		}
		if (url === TROLLEY && method === "PATCH") {
			const { productId, quantity } = body.items[0].actions[0];
			if (refused.includes(productId)) return json(400, { message: "No" });
			if (!ignored.includes(productId)) trolley.set(productId, quantity);
			return json(200, {});
		}
		return json(404, null);
	});
	vi.stubGlobal("fetch", fetchMock);
	return { calls, trolley };
}

/** The Coles page as a logged-in household member sees it. */
function colesPage({
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

const reportOf = (calls: { url: string; body: unknown }[]) =>
	calls.find((c) => c.url.endsWith("/report"))?.body as {
		secret: string;
		delivery: { outcome: string; problem: string };
		lines: { product_id: string; outcome: string; problem?: string }[];
	};

describe("fillColesTrolley", () => {
	beforeEach(() => colesPage());
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
		localStorage.clear();
	});

	it("adds each product on top of the trolley and reports back", async () => {
		const { calls, trolley } = fakeNetwork();

		const result = await fillColesTrolley(config);

		expect(trolley.get("123011")).toBe(3);
		expect(trolley.get("409499")).toBe(1);
		expect(result.added).toEqual(["123011", "409499"]);
		expect(result.failed).toEqual([]);
		expect(result.message).toContain("Added 2 products");
		expect(result.message).toContain("pick a delivery time on Coles");
		const report = reportOf(calls);
		expect(report.secret).toBe("s3cret");
		expect(report.delivery.outcome).toBe("failed");
		expect(report.lines).toEqual([
			{ product_id: "123011", outcome: "added" },
			{ product_id: "409499", outcome: "added" },
		]);
	});

	it("sends the headers the Coles website sends", async () => {
		const { calls } = fakeNetwork();

		await fillColesTrolley(config);

		const patch = calls.find((c) => c.method === "PATCH");
		expect(patch?.headers["ocp-apim-subscription-key"]).toBe("bff-key");
		expect(patch?.headers["cusp-session-id"]).toBe("sess-1");
		expect(patch?.headers["cusp-visitor-id"]).toBe("visit-1");
		expect(patch?.body).toEqual({
			ageGateVerified: false,
			swapBehaviour: false,
			items: [{ actions: [{ productId: "123011", quantity: 3 }] }],
		});
	});

	it("reports a refused or silently ignored product as not added", async () => {
		const { calls } = fakeNetwork({ refused: ["123011"], ignored: ["409499"] });

		const result = await fillColesTrolley(config);

		expect(result.added).toEqual([]);
		expect(reportOf(calls).lines).toEqual([
			{
				product_id: "123011",
				outcome: "failed",
				problem: "Coles answered HTTP 400",
			},
			{
				product_id: "409499",
				outcome: "failed",
				problem:
					"Coles did not add this product. It may be unavailable at your store.",
			},
		]);
	});

	it("refuses a product id that is not a Coles number", async () => {
		const { calls } = fakeNetwork({
			claim: {
				status: 200,
				body: {
					handoff_id: "h1",
					lines: [{ product_id: "w-88436", name: "Milk", quantity: 1 }],
				},
			},
		});

		await fillColesTrolley(config);

		expect(calls.some((c) => c.method === "PATCH")).toBe(false);
		expect(reportOf(calls).lines[0].problem).toBe("Not a Coles product number");
	});

	it("claims nothing until the person is logged in", async () => {
		const { calls } = fakeNetwork({ trolleyStatus: 401 });

		const result = await fillColesTrolley(config);

		expect(result.message).toContain("Log in to Coles");
		expect(calls.some((c) => c.url.endsWith("/claim"))).toBe(false);
	});

	it("claims nothing until a Coles store is chosen", async () => {
		colesPage({ storeId: null });
		const { calls } = fakeNetwork();

		const result = await fillColesTrolley(config);

		expect(result.message).toContain("delivery address");
		expect(calls).toHaveLength(0);
	});

	it("claims nothing while the Coles page is still loading", async () => {
		colesPage({ apiKey: null });
		const { calls } = fakeNetwork();

		const result = await fillColesTrolley(config);

		expect(result.message).toContain("Reload");
		expect(calls).toHaveLength(0);
	});

	it("says so when nothing is waiting", async () => {
		fakeNetwork({ claim: { status: 204 } });

		const result = await fillColesTrolley(config);

		expect(result.handoffId).toBeNull();
		expect(result.message).toContain("Send to Coles");
	});

	it("does nothing on another website", async () => {
		vi.stubGlobal("location", { hostname: "www.woolworths.com.au" });
		const { calls } = fakeNetwork();

		const result = await fillColesTrolley(config);

		expect(result.message).toContain("coles.com.au");
		expect(calls).toHaveLength(0);
	});

	it("stops when the grocery app refuses the secret", async () => {
		fakeNetwork({ claim: { status: 401, body: { detail: "Invalid secret" } } });

		await expect(fillColesTrolley(config)).rejects.toThrow("HTTP 401");
	});
});

describe("the Coles bookmarklet", () => {
	beforeEach(() => colesPage());
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
		localStorage.clear();
	});

	it("is a javascript: link that runs on its own and shows the result", async () => {
		const { calls } = fakeNetwork();
		const alert = vi.fn();
		vi.stubGlobal("alert", alert);

		const href = COLES_TAB.buildBookmarklet({
			...config,
			apiBaseUrl: "https://grocery.test/",
		});

		// Only a test's assertion on our own link, not a URL safety check.
		const [scheme, program] = [href.slice(0, 11), href.slice(11)];
		expect(scheme).toBe("javascript:");
		// Run the link's program exactly as a browser would: nothing from this
		// module is in scope, so it proves the script is self-contained.
		new Function(decodeURIComponent(program))();
		await vi.waitFor(() => expect(alert).toHaveBeenCalled());
		expect(alert.mock.calls[0][0]).toContain("Added 2 products");
		expect(calls.find((c) => c.url.endsWith("/claim"))?.url).toBe(
			"https://grocery.test/api/store-tab/trolley-handoffs/claim",
		);
	});
});
