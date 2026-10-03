/**
 * The products chosen for each list item, one per store at most:
 * `/api/item-selections`, `/api/grocery-items/{id}/selection` and
 * `/api/order-stores`.
 *
 * Only the store and its product id are sent; the backend checks the product
 * against the store's own answer and saves the store's details with it. Money
 * arrives as decimal strings — format it with `lib/money.ts`.
 */
import { request } from "#/lib/api/client";
import type { StoreId, UnitBasis } from "#/lib/api/products";

/** A product chosen for one list item, as the store described it then. */
export interface ItemSelection {
	grocery_item_id: string;
	store: StoreId;
	store_name: string;
	product_id: string;
	name: string;
	brand: string | null;
	package_size: string | null;
	/** Shelf price for one when it was chosen. */
	price: string | null;
	unit_price: { amount: string; per: UnitBasis } | null;
	/** What `priced_quantity` cost when it was chosen, best deal applied. */
	total_price: string | null;
	priced_quantity: number;
	url: string;
	selected_by: string;
	selected_at: string;
	/**
	 * The item's choices at the other stores, which the order does not buy
	 * now. Only in `list()`; the order screen can switch the item to one.
	 */
	also_chosen?: ItemSelection[];
}

/** What the household picked in an item's price comparison. */
export interface ProductChoice {
	store: StoreId;
	product_id: string;
}

/** One item and the store its order buys from. */
export interface OrderStorePick {
	grocery_item_id: string;
	store: StoreId;
}

export const selectionsApi = {
	/** Every item's product to buy, with its choices at the other stores. */
	list: () => request<ItemSelection[]>("/api/item-selections"),
	/**
	 * Chooses the item's product at the choice's store, replacing any earlier
	 * choice there, and makes it the one the order buys.
	 */
	choose: (itemId: string, choice: ProductChoice) =>
		request<ItemSelection>(`/api/grocery-items/${itemId}/selection`, {
			method: "PUT",
			body: JSON.stringify(choice),
		}),
	/**
	 * Forgets the item's product at `store`, or at every store. Clearing the
	 * one to buy hands that role to the item's other choice.
	 */
	clear: (itemId: string, store?: StoreId) =>
		request<void>(
			`/api/grocery-items/${itemId}/selection${store ? `?store=${store}` : ""}`,
			{ method: "DELETE" },
		),
	/** Makes each item's choice at the paired store the one the order buys. */
	buyAt: (picks: OrderStorePick[]) =>
		request<void>("/api/order-stores", {
			method: "PUT",
			body: JSON.stringify({ picks }),
		}),
};

/** The chosen products keyed by the item they were chosen for. */
export function selectionsByItem(
	selections: ItemSelection[],
): Map<string, ItemSelection> {
	return new Map(selections.map((s) => [s.grocery_item_id, s]));
}
