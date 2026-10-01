/**
 * The order review: `GET /api/order-review`.
 *
 * Every committed list item, its chosen product re-priced at its store now,
 * grouped by store. Money arrives as decimal strings — format it with
 * `lib/money.ts`. `chosen_*` fields are what the store showed when the
 * product was chosen; the others are today's.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";

/** Whether a chosen product can be bought as it is. */
export type OrderLineStatus =
	| "priced"
	| "unavailable"
	| "not_offered"
	| "store_failed";

/** Shelf price for one, today against when it was chosen. */
export type PriceChange = "same" | "up" | "down";

/** One item of the order. */
export interface OrderLine {
	grocery_item_id: string;
	item_name: string;
	/** The item's quantity now — what will be bought. */
	quantity: number;
	product_id: string;
	product_name: string;
	brand: string | null;
	package_size: string | null;
	url: string;
	status: OrderLineStatus;
	/** Shelf price for one today. */
	price: string | null;
	/** `quantity` today, best deal applied. */
	total_price: string | null;
	deal_applied: boolean;
	price_change: PriceChange | null;
	chosen_price: string | null;
	chosen_total_price: string | null;
	chosen_priced_quantity: number;
	/** A sentence to show when the line cannot be bought as it is. */
	problem: string | null;
}

/** Everything bought at one store. */
export interface StoreOrder {
	store: StoreId;
	store_name: string;
	lines: OrderLine[];
	subtotal: string;
	/** Every line has a price today. */
	complete: boolean;
}

/** A committed item still waiting for its product. */
export interface UnchosenItem {
	grocery_item_id: string;
	name: string;
	quantity: number;
}

/** The order as it stands right now. */
export interface OrderReview {
	/** Stores with something to buy, Woolworths first. */
	stores: StoreOrder[];
	unchosen: UnchosenItem[];
	/** Item costs only — delivery fees are not included. */
	total: string;
	/** Every item has a product with a price today. */
	complete: boolean;
}

export const orderReviewApi = {
	/** Re-prices the committed list now. */
	get: () => request<OrderReview>("/api/order-review"),
};

/** Nothing is committed, so there is nothing to order. */
export function isEmptyOrder(review: OrderReview): boolean {
	return review.stores.length === 0 && review.unchosen.length === 0;
}
