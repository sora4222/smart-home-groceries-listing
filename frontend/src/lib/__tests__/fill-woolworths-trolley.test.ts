import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { buildFillWoolworthsTrolleyBookmarklet } from "#/lib/store-tab/bookmarklet";
import { fillWoolworthsTrolley } from "#/lib/store-tab/fill-woolworths-trolley";

const config = { apiBaseUrl: "https://grocery.test", secret: "s3cret" };

/** A fake network: our backend plus the Woolworths website's own calls. */
function fakeNetwork({
	claim = {
		status: 200,
		body: {
			handoff_id: "h1",
			store: "woolworths",
			lines: [
				{ product_id: "88436", name: "Milk 2L", quantity: 2 },
				{ product_id: "111", name: "Bread", quantity: 1 },
			],
		},
	},
	inTrolley = [{ Stockcode: 88436, QuantityInTrolley: 1 }],
	unknown = ["111"],
	unavailable = [],
}: {
	claim?: { status: number; body?: unknown };
	inTrolley?: { Stockcode: number; QuantityInTrolley: number }[];
	unknown?: string[];
	unavailable?: string[];
} = {}) {
	const calls: { url: string; body: unknown }[] = [];
	const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
		const body = init?.body ? JSON.parse(String(init.body)) : undefined;
		calls.push({ url, body });
		const json = (status: number, value: unknown) =>
			new Response(value === undefined ? null : JSON.stringify(value), {
				status,
			});
		if (url.endsWith("/claim")) return json(claim.status, claim.body);
		if (url.endsWith("/report")) return json(200, {});
		if (url === "/api/v3/ui/trolley") return json(200, { Products: inTrolley });
		if (url === "/api/v3/ui/trolley/update") {
			// As the live site answers: an unknown stockcode updates nothing.
			const item = body.items[0];
			if (unknown.includes(String(item.stockcode))) {
				return json(200, { Message: null, UpdatedItems: [] });
			}
			return json(200, {
				UpdatedItems: [
					{
						Stockcode: item.stockcode,
						IsAvailable: !unavailable.includes(String(item.stockcode)),
						QuantityInTrolley: item.quantity,
					},
				],
			});
		}
		return json(404, null);
	});
	vi.stubGlobal("fetch", fetchMock);
	return calls;
}

describe("fillWoolworthsTrolley", () => {
	beforeEach(() => {
		vi.stubGlobal("location", { hostname: "www.woolworths.com.au" });
	});
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("adds each product on top of the trolley and reports every outcome", async () => {
		const calls = fakeNetwork();

		const result = await fillWoolworthsTrolley(config);

		expect(calls[0]).toEqual({
			url: "https://grocery.test/api/store-tab/trolley-handoffs/claim",
			body: { store: "woolworths", secret: "s3cret" },
		});
		const updates = calls.filter((c) => c.url === "/api/v3/ui/trolley/update");
		// 1 milk already in the trolley + 2 from the list.
		expect(
			updates.map(
				(u) =>
					(u.body as { items: { stockcode: number; quantity: number }[] })
						.items[0],
			),
		).toMatchObject([
			{ stockcode: 88436, quantity: 3 },
			{ stockcode: 111, quantity: 1 },
		]);
		const report = calls.at(-1);
		expect(report?.url).toBe(
			"https://grocery.test/api/store-tab/trolley-handoffs/h1/report",
		);
		expect(report?.body).toEqual({
			secret: "s3cret",
			lines: [
				{ product_id: "88436", outcome: "added" },
				{
					product_id: "111",
					outcome: "failed",
					problem: "Woolworths did not add this product",
				},
			],
		});
		expect(result.added).toEqual(["88436"]);
		expect(result.message).toContain("1 could not be added");
	});

	it("counts a product in the trolley as added even when Woolworths flags it unavailable", async () => {
		// Live behaviour, 2026-10-02: with no delivery time picked, a logged-in
		// trolley answers IsAvailable false and $0 for products it did add.
		const calls = fakeNetwork({ unknown: [], unavailable: ["88436"] });

		const result = await fillWoolworthsTrolley(config);

		expect(result.added).toEqual(["88436", "111"]);
		const report = calls.at(-1)?.body as {
			lines: { product_id: string; outcome: string; problem?: string }[];
		};
		expect(report.lines[0].outcome).toBe("added");
		expect(report.lines[0].problem).toContain("pick a delivery time");
		expect(result.message).toContain("Added 2 products");
	});

	it("does nothing when no handoff is waiting", async () => {
		const calls = fakeNetwork({ claim: { status: 204 } });

		const result = await fillWoolworthsTrolley(config);

		expect(result.handoffId).toBeNull();
		expect(result.message).toContain("Send to Woolworths");
		expect(calls).toHaveLength(1);
	});

	it("refuses to run anywhere but Woolworths", async () => {
		vi.stubGlobal("location", { hostname: "evil.example" });
		const calls = fakeNetwork();

		const result = await fillWoolworthsTrolley(config);

		expect(result.message).toContain("woolworths.com.au");
		expect(calls).toHaveLength(0);
	});

	it("stops when the grocery app refuses the secret", async () => {
		fakeNetwork({ claim: { status: 401, body: { detail: "Invalid secret" } } });

		await expect(fillWoolworthsTrolley(config)).rejects.toThrow("HTTP 401");
	});
});

describe("buildFillWoolworthsTrolleyBookmarklet", () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("is a javascript: link that runs on its own and shows the result", async () => {
		vi.stubGlobal("location", { hostname: "www.woolworths.com.au" });
		const calls = fakeNetwork({ claim: { status: 204 } });
		const alert = vi.fn();
		vi.stubGlobal("alert", alert);

		const href = buildFillWoolworthsTrolleyBookmarklet({
			...config,
			apiBaseUrl: "https://grocery.test/",
		});

		expect(href.startsWith("javascript:")).toBe(true);
		// Run the link's program exactly as a browser would: nothing from this
		// module is in scope, so it proves the script is self-contained.
		new Function(decodeURIComponent(href.slice("javascript:".length)))();
		await vi.waitFor(() => expect(alert).toHaveBeenCalled());
		expect(alert.mock.calls[0][0]).toContain("Nothing to add");
		expect(calls[0].url).toBe(
			"https://grocery.test/api/store-tab/trolley-handoffs/claim",
		);
	});
});
