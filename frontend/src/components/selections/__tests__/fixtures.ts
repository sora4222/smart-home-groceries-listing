import type { ItemSelection } from "#/lib/api";

/** A saved choice for the shared milk item; tests override what they test. */
export function selectionFixture(
	overrides: Partial<ItemSelection> = {},
): ItemSelection {
	return {
		grocery_item_id: "33333333-3333-3333-3333-333333333333",
		store: "coles",
		store_name: "Coles",
		product_id: "c-milk-3l",
		name: "Full Cream Milk",
		brand: "Coles",
		package_size: "3L",
		price: "4.95",
		unit_price: { amount: "0.165", per: "100mL" },
		total_price: "9.9",
		priced_quantity: 2,
		url: "https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288",
		selected_by: "user_dev",
		selected_at: new Date("2026-10-01T00:00:00Z").toISOString(),
		...overrides,
	};
}
