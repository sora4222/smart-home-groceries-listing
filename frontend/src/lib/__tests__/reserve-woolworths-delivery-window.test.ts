import { afterEach, describe, expect, it, vi } from "vitest";
import { deliveryInfo, liveDays } from "#/lib/__tests__/woolworths-fixtures";
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";
import { reserveWoolworthsDeliveryWindow } from "#/lib/store-tab/reserve-woolworths-delivery-window";

/** A fake Woolworths for the three delivery calls. */
function fakeWoolworths({
	info = deliveryInfo() as unknown,
	days = liveDays(),
	reserveAnswer = { IsSuccessful: true, Message: null } as unknown,
} = {}) {
	const calls: { url: string; body?: unknown }[] = [];
	vi.stubGlobal(
		"fetch",
		vi.fn(async (url: string, init?: RequestInit) => {
			calls.push({
				url,
				body: init?.body ? JSON.parse(String(init.body)) : undefined,
			});
			const json = (value: unknown) =>
				new Response(JSON.stringify(value), { status: 200 });
			if (url === "/apis/ui/Delivery/DeliveryInfo") return json(info);
			if (url.startsWith("/api/v3/ui/fulfilment/windows"))
				return json({ Days: days });
			if (url === "/apis/ui/Fulfilment") return json(reserveAnswer);
			return new Response(null, { status: 404 });
		}),
	);
	return calls;
}

describe("reserveWoolworthsDeliveryWindow", () => {
	afterEach(() => vi.unstubAllGlobals());

	it("reserves the next day's window with the website's own call", async () => {
		const calls = fakeWoolworths();

		const report = await reserveWoolworthsDeliveryWindow(
			{ date: null, time_of_day: "morning" },
			chooseWoolworthsWindow,
		);

		expect(calls[1].url).toBe(
			"/api/v3/ui/fulfilment/windows?areaId=6145&fulfilmentMethod=Courier&addressId=35335206",
		);
		expect(calls[2]).toEqual({
			url: "/apis/ui/Fulfilment",
			body: {
				addressId: 35335206,
				fulfilmentMethod: "Courier",
				timeslotId: 948311,
				windowDate: "2026-10-03",
			},
		});
		expect(report).toEqual({
			outcome: "reserved",
			window_label: "4am - 7am",
			window_start: "2026-10-03T04:00:00",
			window_end: "2026-10-03T07:00:00",
			fee: "15",
			problem: undefined,
		});
	});

	it("keeps a window already reserved that fits, without changing it", async () => {
		const calls = fakeWoolworths({
			info: deliveryInfo({
				id: 948284,
				start: "2026-10-03T07:00:00.0000000",
				end: "2026-10-03T10:00:00.0000000",
				label: "7:00am - 10:00am",
			}),
		});

		const report = await reserveWoolworthsDeliveryWindow(
			{ date: null, time_of_day: "morning" },
			chooseWoolworthsWindow,
		);

		expect(report.outcome).toBe("kept");
		expect(report.window_label).toBe("7:00am - 10:00am");
		expect(calls).toHaveLength(1);
	});

	it("replaces a reserved window that does not fit what was asked", async () => {
		fakeWoolworths({
			info: deliveryInfo({
				id: 948284,
				start: "2026-10-03T07:00:00.0000000",
				end: "2026-10-03T10:00:00.0000000",
				label: "7:00am - 10:00am",
			}),
		});

		const report = await reserveWoolworthsDeliveryWindow(
			{ date: null, time_of_day: "evening" },
			chooseWoolworthsWindow,
		);

		expect(report.outcome).toBe("reserved");
		expect(report.window_start).toBe("2026-10-03T17:00:00");
	});

	it("explains when it cannot reserve", async () => {
		fakeWoolworths({ info: { ...deliveryInfo(), Address: null } });
		expect(
			(
				await reserveWoolworthsDeliveryWindow(
					{ date: null, time_of_day: "any" },
					chooseWoolworthsWindow,
				)
			).problem,
		).toContain("delivery address");

		fakeWoolworths({ days: [] });
		expect(
			(
				await reserveWoolworthsDeliveryWindow(
					{ date: null, time_of_day: "evening" },
					chooseWoolworthsWindow,
				)
			).problem,
		).toContain("no evening delivery times");

		fakeWoolworths({
			reserveAnswer: { IsSuccessful: false, Message: "Window full" },
		});
		const full = await reserveWoolworthsDeliveryWindow(
			{ date: null, time_of_day: "any" },
			chooseWoolworthsWindow,
		);
		expect(full).toEqual({ outcome: "failed", problem: "Window full" });
	});
});
