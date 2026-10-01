/**
 * Woolworths answers, trimmed from the live site on 2026-10-02 (store 3800,
 * a Melbourne address). Personal details are removed.
 */
import type {
	WoolworthsDay,
	WoolworthsWindow,
} from "#/lib/store-tab/choose-woolworths-window";

export function windowAt(
	id: number,
	date: string,
	startHour: number,
	hours: number,
	price = 15,
	extra: Partial<WoolworthsWindow> = {},
): WoolworthsWindow {
	const pad = (n: number) => String(n).padStart(2, "0");
	const label = (h: number) => `${h % 12 || 12}${h < 12 ? "am" : "pm"}`;
	return {
		Id: id,
		Available: true,
		TimeWindow: `${label(startHour)} - ${label(startHour + hours)}`,
		StartDateTime: `${date}T${pad(startHour)}:00:00.0000000`,
		EndDateTime: `${date}T${pad(startHour + hours)}:00:00.0000000`,
		SalePrice: price,
		TimeWindowDurationHours: hours,
		IsExpress: false,
		...extra,
	};
}

/** Today (2 Oct) and tomorrow (3 Oct), as the windows call lists them. */
export function liveDays(): WoolworthsDay[] {
	return [
		{
			Date: "2026-10-02T00:00:00.0000000",
			Available: true,
			Times: [
				windowAt(1078689, "2026-10-02", 11, 1),
				windowAt(948276, "2026-10-02", 14, 3),
			],
		},
		{
			Date: "2026-10-03T00:00:00.0000000",
			Available: true,
			Times: [
				windowAt(948311, "2026-10-03", 4, 3),
				windowAt(948284, "2026-10-03", 7, 3),
				windowAt(948304, "2026-10-03", 12, 5),
				windowAt(948288, "2026-10-03", 13, 3),
				windowAt(948300, "2026-10-03", 17, 3),
			],
		},
	];
}

/** `GET /apis/ui/Delivery/DeliveryInfo`, with or without a reserved window. */
export function deliveryInfo(
	reserved: {
		id: number;
		start: string;
		end: string;
		label: string;
	} | null = null,
) {
	return {
		DeliveryMethod: "Courier",
		Address: {
			AddressId: 35335206,
			AreaId: 6145,
			PostalCode: "3124",
			State: "VIC",
		},
		CurrentDateAtFulfilmentStore: "2026-10-02T00:00:00.0000000",
		FulfilmentStoreId: 3800,
		ReservedTime: reserved
			? {
					Id: reserved.id,
					DeliveryWindowId: reserved.id,
					StartDateTime: reserved.start,
					EndDateTime: reserved.end,
					TimeWindow: reserved.label,
				}
			: {
					Id: 0,
					DeliveryWindowId: 0,
					StartDateTime: null,
					EndDateTime: null,
					TimeWindow: null,
				},
	};
}
