import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { deliveryInfo, liveDays } from "#/lib/__tests__/woolworths-fixtures";
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";
import { fillWoolworthsTrolley } from "#/lib/store-tab/fill-woolworths-trolley";
import { reserveWoolworthsDeliveryWindow } from "#/lib/store-tab/reserve-woolworths-delivery-window";
import { WOOLWORTHS_TAB } from "#/lib/store-tab/store-tabs";

const config = { apiBaseUrl: "https://grocery.test", secret: "s3cret" };
const helpers = {
	chooseWindow: chooseWoolworthsWindow,
	reserveDeliveryWindow: reserveWoolworthsDeliveryWindow,
};
const fill = () => fillWoolworthsTrolley(config, helpers);

/** A fake network: our backend plus the Woolworths website's own calls. */
function fakeNetwork({
	claim = {
		status: 200,
		body: {
			handoff_id: "h1",
			store: "woolworths",
			delivery: { date: null, time_of_day: "morning" },
			lines: [
				{ product_id: "88436", name: "Milk 2L", quantity: 2 },
				{ product_id: "111", name: "Bread", quantity: 1 },
			],
		},
	},
	inTrolley = [{ Stockcode: 88436, QuantityInTrolley: 1 }],
	unknown = ["111"],
	unavailable = [] as string[],
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
		if (url === "/apis/ui/Delivery/DeliveryInfo")
			return json(200, deliveryInfo());
		if (url.startsWith("/api/v3/ui/fulfilment/windows"))
			return json(200, { Days: liveDays() });
		if (url === "/apis/ui/Fulfilment") return json(200, { IsSuccessful: true });
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

		const result = await fill();

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
			delivery: {
				outcome: "reserved",
				window_label: "4am - 7am",
				window_start: "2026-10-03T04:00:00",
				window_end: "2026-10-03T07:00:00",
				fee: "15",
			},
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
		expect(result.message).toContain("Delivery: 2026-10-03, 4am - 7am");
	});

	it("reserves the delivery time before adding any product", async () => {
		const calls = fakeNetwork();

		await fill();

		const order = calls.map((c) => c.url);
		expect(order.indexOf("/apis/ui/Fulfilment")).toBeLessThan(
			order.indexOf("/api/v3/ui/trolley/update"),
		);
	});

	it("takes a product unavailable at the household's store back out, and says so", async () => {
		// Live, 2026-10-02: the trolley call adds it but answers IsAvailable
		// false and $0 — Woolworths' own product page says unavailable too.
		const calls = fakeNetwork({ unknown: [], unavailable: ["88436"] });

		const result = await fill();

		const milkUpdates = calls
			.filter((c) => c.url === "/api/v3/ui/trolley/update")
			.map(
				(c) =>
					(c.body as { items: { stockcode: number; quantity: number }[] })
						.items[0],
			)
			.filter((item) => item.stockcode === 88436);
		// Raised from 1 to 3, then put back to the 1 that was already there.
		expect(milkUpdates.map((i) => i.quantity)).toEqual([3, 1]);
		expect(result.added).toEqual(["111"]);
		expect(result.failed).toEqual([
			{
				productId: "88436",
				problem: "Not available at your Woolworths store right now",
			},
		]);
	});

	it("does nothing when no handoff is waiting", async () => {
		const calls = fakeNetwork({ claim: { status: 204 } });

		const result = await fill();

		expect(result.handoffId).toBeNull();
		expect(result.message).toContain("Send to Woolworths");
		expect(calls).toHaveLength(1);
	});

	it("refuses to run anywhere but Woolworths", async () => {
		vi.stubGlobal("location", { hostname: "evil.example" });
		const calls = fakeNetwork();

		const result = await fill();

		expect(result.message).toContain("woolworths.com.au");
		expect(calls).toHaveLength(0);
	});

	it("stops when the grocery app refuses the secret", async () => {
		fakeNetwork({ claim: { status: 401, body: { detail: "Invalid secret" } } });

		await expect(fill()).rejects.toThrow("HTTP 401");
	});
});

describe("the Woolworths bookmarklet", () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("is a javascript: link that runs on its own and shows the result", async () => {
		vi.stubGlobal("location", { hostname: "www.woolworths.com.au" });
		const calls = fakeNetwork();
		const alert = vi.fn();
		vi.stubGlobal("alert", alert);

		const href = WOOLWORTHS_TAB.buildBookmarklet({
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
		expect(alert.mock.calls[0][0]).toContain("Delivery: 2026-10-03, 4am - 7am");
		expect(calls.some((c) => c.url === "/apis/ui/Fulfilment")).toBe(true);
		expect(calls[0].url).toBe(
			"https://grocery.test/api/store-tab/trolley-handoffs/claim",
		);
	});
});
