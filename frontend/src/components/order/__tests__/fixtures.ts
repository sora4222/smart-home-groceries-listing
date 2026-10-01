import type { OrderLine, OrderReview, StoreOrder } from "#/lib/api";

/** Coles milk, priced today as when it was chosen; tests override the rest. */
export function lineFixture(overrides: Partial<OrderLine> = {}): OrderLine {
	return {
		grocery_item_id: "33333333-3333-3333-3333-333333333333",
		item_name: "milk",
		quantity: 2,
		product_id: "c-milk-3l",
		product_name: "Full Cream Milk",
		brand: "Coles",
		package_size: "3L",
		url: "https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288",
		status: "priced",
		price: "4.95",
		total_price: "9.90",
		deal_applied: false,
		price_change: "same",
		chosen_price: "4.95",
		chosen_total_price: "9.90",
		chosen_priced_quantity: 2,
		problem: null,
		...overrides,
	};
}

/** One store's part of the order, holding `lines`. */
export function storeFixture(overrides: Partial<StoreOrder> = {}): StoreOrder {
	return {
		store: "coles",
		store_name: "Coles",
		lines: [lineFixture()],
		subtotal: "9.90",
		complete: true,
		...overrides,
	};
}

/** An order with one Woolworths line and one Coles line, all priced. */
export function reviewFixture(
	overrides: Partial<OrderReview> = {},
): OrderReview {
	return {
		stores: [
			storeFixture({
				store: "woolworths",
				store_name: "Woolworths",
				subtotal: "4.00",
				lines: [
					lineFixture({
						grocery_item_id: "44444444-4444-4444-4444-444444444444",
						item_name: "bananas",
						quantity: 5,
						product_id: "w-banana",
						product_name: "Cavendish Bananas",
						brand: null,
						package_size: "each",
						price: "0.80",
						total_price: "4.00",
						chosen_price: "0.80",
					}),
				],
			}),
			storeFixture(),
		],
		unchosen: [],
		total: "13.90",
		complete: true,
		...overrides,
	};
}
