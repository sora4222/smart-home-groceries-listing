import type { ItemProducts, StoreProduct, StoreProducts } from "#/lib/api";

/** A plain, available product; each test overrides what it is about. */
export function productFixture(
	overrides: Partial<StoreProduct> = {},
): StoreProduct {
	return {
		product_id: "c-milk-3l",
		name: "Full Cream Milk",
		brand: "Coles",
		package_size: "3L",
		price: "4.95",
		was_price: null,
		on_special: false,
		unit_price: { amount: "0.165", per: "100mL", converted_from: "1L" },
		unit_price_note: "unit price calculated from 1L",
		deals: [],
		total_price: "4.95",
		deal_applied: false,
		category: "Milk",
		url: "https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288",
		available: true,
		...overrides,
	};
}

/** One store's answer. */
export function storeFixture(
	overrides: Partial<StoreProducts> = {},
): StoreProducts {
	return {
		store: "coles",
		store_name: "Coles",
		status: "ok",
		message: null,
		products: [productFixture()],
		...overrides,
	};
}

/** A whole search for one item. */
export function searchFixture(stores: StoreProducts[]): ItemProducts {
	return {
		item: {
			id: "33333333-3333-3333-3333-333333333333",
			name: "milk",
			quantity: 2,
			filter_terms: [],
		},
		query: "milk",
		stores,
	};
}
