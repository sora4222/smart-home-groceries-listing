/**
 * Reserves a Woolworths delivery window from inside the household's
 * logged-in woolworths.com.au tab, before any product is added.
 *
 * Calls (live, 2026-10-02):
 * - `GET /apis/ui/Delivery/DeliveryInfo` — the account's address (`AddressId`,
 *   `AreaId`), method (`Courier` = delivery), the store's "today", and the
 *   window already reserved (`ReservedTime`).
 * - `GET /api/v3/ui/fulfilment/windows?areaId&fulfilmentMethod&addressId` —
 *   `Days[].Times[]` for about a week.
 * - `POST /apis/ui/Fulfilment` `{addressId, fulfilmentMethod, timeslotId,
 *   windowDate}` — reserves one; answers `{IsSuccessful}`.
 *
 * A window already reserved on the right day and part of the day is kept, so
 * a time the household picked on Woolworths is not overwritten. The window
 * can always be changed on Woolworths later.
 *
 * Must stay **self-contained**: the bookmarklet carries its source text, and
 * the window rule arrives as the `chooseWindow` argument
 * (`chooseWoolworthsWindow`).
 */
import type {
	ChosenWindow,
	DeliveryWanted,
	WoolworthsDay,
} from "#/lib/store-tab/choose-woolworths-window";

/** What happened to the delivery window, in the backend's report shape. */
export interface DeliveryReport {
	outcome: "reserved" | "kept" | "failed";
	window_label?: string;
	/** Store-local, "YYYY-MM-DDTHH:MM:SS". */
	window_start?: string;
	window_end?: string;
	/** Decimal string. */
	fee?: string;
	problem?: string;
}

type ChooseWindow = (
	days: WoolworthsDay[],
	wanted: DeliveryWanted,
	storeToday: string,
) => ChosenWindow | null;

/** Reserves (or keeps) the delivery window `wanted` asks for. Never throws. */
export async function reserveWoolworthsDeliveryWindow(
	wanted: DeliveryWanted,
	chooseWindow: ChooseWindow,
): Promise<DeliveryReport> {
	const JSON_HEADERS = {
		accept: "application/json, text/plain, */*",
		"content-type": "application/json",
	};
	const seconds = (dateTime: string) => dateTime.slice(0, 19);
	const failed = (problem: string): DeliveryReport => ({
		outcome: "failed",
		problem: problem.slice(0, 300),
	});
	try {
		const info = await fetch("/apis/ui/Delivery/DeliveryInfo", {
			credentials: "include",
			headers: JSON_HEADERS,
			cache: "no-store",
		}).then((r) => (r.ok ? r.json() : null));
		const address = info?.Address;
		if (!address?.AddressId) {
			return failed("Add a delivery address on Woolworths first.");
		}
		const method = info.DeliveryMethod || "Courier";
		const storeToday = String(info.CurrentDateAtFulfilmentStore ?? "").slice(
			0,
			10,
		);

		const reserved = info.ReservedTime;
		if (reserved?.Id && reserved.StartDateTime) {
			const asDay = [
				{
					Date: reserved.StartDateTime,
					Available: true,
					Times: [
						{
							Id: reserved.Id,
							Available: true,
							TimeWindow: reserved.TimeWindow ?? "",
							StartDateTime: reserved.StartDateTime,
							EndDateTime: reserved.EndDateTime,
							SalePrice: 0,
						},
					],
				},
			];
			const fits = chooseWindow(asDay, wanted, storeToday);
			if (fits && !fits.movedToLaterDay) {
				return {
					outcome: "kept",
					window_label: reserved.TimeWindow,
					window_start: seconds(reserved.StartDateTime),
					window_end: seconds(reserved.EndDateTime),
				};
			}
		}

		const query = new URLSearchParams({
			areaId: String(address.AreaId),
			fulfilmentMethod: method,
			addressId: String(address.AddressId),
		});
		const listed = await fetch(`/api/v3/ui/fulfilment/windows?${query}`, {
			credentials: "include",
			headers: JSON_HEADERS,
			cache: "no-store",
		}).then((r) => (r.ok ? r.json() : null));
		const chosen = chooseWindow(listed?.Days ?? [], wanted, storeToday);
		if (!chosen) {
			return failed(
				wanted.time_of_day === "any"
					? "Woolworths has no delivery times left this week."
					: `Woolworths has no ${wanted.time_of_day} delivery times left this week.`,
			);
		}

		const answer = await fetch("/apis/ui/Fulfilment", {
			method: "POST",
			credentials: "include",
			headers: JSON_HEADERS,
			body: JSON.stringify({
				addressId: address.AddressId,
				fulfilmentMethod: method,
				timeslotId: chosen.window.Id,
				windowDate: chosen.date,
			}),
		}).then((r) => (r.ok ? r.json() : null));
		if (!answer?.IsSuccessful) {
			return failed(
				answer?.Message || "Woolworths did not reserve the delivery time.",
			);
		}
		return {
			outcome: "reserved",
			window_label: chosen.window.TimeWindow,
			window_start: seconds(chosen.window.StartDateTime),
			window_end: seconds(chosen.window.EndDateTime),
			fee: String(chosen.window.SalePrice),
			problem: chosen.movedToLaterDay
				? `Nothing left on ${chosen.wantedDate}, so the next day with a time was used.`
				: undefined,
		};
	} catch (error) {
		return failed(`Could not reserve a delivery time: ${String(error)}`);
	}
}
