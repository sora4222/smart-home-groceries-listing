/**
 * The products chosen for each list item:
 * `/api/item-selections`, `/api/grocery-items/{id}/selection`,
 * `/api/selection-snapshots` and `/api/order-stores`.
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
	/** The product's picture when it was chosen; older choices have none. */
	image_url: string | null;
	selected_by: string;
	selected_at: string;
	/** How many of this product the order buys. */
	quantity: number;
}

/** One item's choices, as `list()` returns them. */
export interface ItemChoices extends ItemSelection {
	/**
	 * More products the order buys for the item, at the same store. The
	 * counts of all of them add up to the item's quantity.
	 */
	bought_with: ItemSelection[];
	/**
	 * The item's choices at the other stores, which the order does not buy
	 * now. The order screen can switch the item to one.
	 */
	also_chosen: ItemSelection[];
}

/** Every product chosen for the item: the ones the order buys, then the rest. */
export function everyChoice(choices: ItemChoices): ItemSelection[] {
	return [choices, ...choices.bought_with, ...choices.also_chosen];
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
	list: () => request<ItemChoices[]>("/api/item-selections"),
	/**
	 * Chooses a product for the item and makes its store the one the order
	 * buys from. An item wanting one swaps its product; an item wanting more
	 * adds it with a count of 1 (409 when it has as many as it wants).
	 */
	choose: (itemId: string, choice: ProductChoice) =>
		request<ItemSelection>(`/api/grocery-items/${itemId}/selection`, {
			method: "PUT",
			body: JSON.stringify(choice),
		}),
	/**
	 * Forgets one product (`productId`), every product at `store`, or every
	 * choice. Clearing the ones to buy hands that role to the other store.
	 */
	clear: (itemId: string, store?: StoreId, productId?: string) => {
		const query = new URLSearchParams();
		if (store) query.set("store", store);
		if (store && productId) query.set("product_id", productId);
		const search = query.size > 0 ? `?${query}` : "";
		return request<void>(`/api/grocery-items/${itemId}/selection${search}`, {
			method: "DELETE",
		});
	},
	/**
	 * How many of one chosen product to buy; the item's other products at
	 * the store share the rest. 422 when that leaves one of them none.
	 */
	setQuantity: (
		itemId: string,
		store: StoreId,
		productId: string,
		quantity: number,
	) =>
		request<void>(`/api/grocery-items/${itemId}/selection/quantity`, {
			method: "PUT",
			body: JSON.stringify({ store, product_id: productId, quantity }),
		}),
	/** Saves the item's products before a change, for Undo. */
	snapshot: (itemId: string) =>
		request<{ id: string }>(`/api/grocery-items/${itemId}/selection/snapshot`, {
			method: "POST",
		}),
	/**
	 * Undo: puts a snapshot's products back in one step, with no store
	 * search. `itemId` names the item when it is not the one the snapshot
	 * was taken of (a removed item re-added with a new id).
	 */
	restore: (snapshotId: string, itemId?: string) => {
		const search = itemId ? `?grocery_item_id=${itemId}` : "";
		return request<void>(
			`/api/selection-snapshots/${snapshotId}/restore${search}`,
			{ method: "POST" },
		);
	},
	/** Makes each item's choice at the paired store the one the order buys. */
	buyAt: (picks: OrderStorePick[]) =>
		request<void>("/api/order-stores", {
			method: "PUT",
			body: JSON.stringify({ picks }),
		}),
};

/** The chosen products keyed by the item they were chosen for. */
export function selectionsByItem(
	selections: ItemChoices[],
): Map<string, ItemChoices> {
	return new Map(selections.map((s) => [s.grocery_item_id, s]));
}
