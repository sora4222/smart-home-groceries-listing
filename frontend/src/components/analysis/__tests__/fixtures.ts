import type { PurchaseOrder, SpendingReport } from "#/lib/api";

/** A saved Woolworths shop of two products. */
export function orderFixture(
	overrides: Partial<PurchaseOrder> = {},
): PurchaseOrder {
	return {
		id: "order-1",
		store: "woolworths",
		store_name: "Woolworths",
		source: "trolley_fill",
		trolley_handoff_id: "handoff-1",
		bought_at: "2026-10-02T01:00:00Z",
		items_total: "18.20",
		delivery_fee: "15",
		total: "33.20",
		products: 2,
		...overrides,
	};
}

/** Spending with one of everything. */
export function reportFixture(
	overrides: Partial<SpendingReport> = {},
): SpendingReport {
	return {
		latest_order: orderFixture(),
		categories: ["Bakery", "Dairy & eggs"],
		items_total: "12.05",
		delivery_total: "2",
		total: "14.05",
		over_time: [
			{ start: "2026-07-01", total: "10.95" },
			{ start: "2026-08-01", total: "0" },
			{ start: "2026-09-01", total: "3.10" },
		],
		by_item: [
			{ item_name: "milk", quantity: 2, times_bought: 2, total: "8.05" },
			{ item_name: "Bread", quantity: 1, times_bought: 1, total: "4" },
		],
		by_store: [
			{
				store: "woolworths",
				store_name: "Woolworths",
				items_total: "7.10",
				delivery_total: "0",
				total: "7.10",
			},
			{
				store: "coles",
				store_name: "Coles",
				items_total: "4.95",
				delivery_total: "2",
				total: "6.95",
			},
		],
		by_category: [
			{ category: "Dairy & eggs", total: "8.05" },
			{ category: "Bakery", total: "4" },
		],
		...overrides,
	};
}
