/**
 * A list item's products at each store: `/api/grocery-items/{id}/products`.
 *
 * Money arrives as decimal strings (`"4.95"`) so nothing is lost to
 * floating point on the way; format it with `lib/money.ts`.
 */
import { request } from "#/lib/api/client";

export type StoreId = "woolworths" | "coles";

/** `ok`, or why the store produced nothing. */
export type StoreStatus =
	| "ok"
	| "blocked"
	| "unreachable"
	| "unexpected_response";

/** The common basis a unit price is stated against. */
export type UnitBasis = "100g" | "100mL" | "unit";

export interface UnitPrice {
	amount: string;
	per: UnitBasis;
	/** The store's own measure, when converting from it changed the basis. */
	converted_from: string | null;
}

/** A multibuy: `unit_price` each once `min_quantity` are bought. */
export interface Deal {
	description: string;
	min_quantity: number;
	unit_price: string;
}

export interface StoreProduct {
	product_id: string;
	name: string;
	brand: string | null;
	package_size: string | null;
	/** Shelf price for one; `null` when the store shows none. */
	price: string | null;
	was_price: string | null;
	on_special: boolean;
	unit_price: UnitPrice | null;
	/** Why the unit price needs care: converted, other units, missing. */
	unit_price_note: string | null;
	deals: Deal[];
	/** What the item's quantity costs, best deal applied. */
	total_price: string | null;
	deal_applied: boolean;
	category: string | null;
	url: string;
	available: boolean;
}

export interface StoreProducts {
	store: StoreId;
	store_name: string;
	status: StoreStatus;
	/** A sentence to show when the store failed. */
	message: string | null;
	/** Cheapest per unit first. */
	products: StoreProduct[];
}

export interface ItemProducts {
	item: {
		id: string;
		name: string;
		quantity: number;
		filter_terms: string[];
	};
	/** What each store was asked for: the name and chips together. */
	query: string;
	stores: StoreProducts[];
}

export const productsApi = {
	/** Searches every store for one list item. */
	forItem: (itemId: string, init?: RequestInit) =>
		request<ItemProducts>(`/api/grocery-items/${itemId}/products`, init),
};
