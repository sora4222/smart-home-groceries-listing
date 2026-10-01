/**
 * The one product chosen for each list item: `/api/item-selections` and
 * `/api/grocery-items/{id}/selection`.
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
}

/** What the household picked in an item's price comparison. */
export interface ProductChoice {
	store: StoreId;
	product_id: string;
}

export const selectionsApi = {
	/** Every item's chosen product. */
	list: () => request<ItemSelection[]>("/api/item-selections"),
	/** Chooses the item's product, replacing any earlier choice. */
	choose: (itemId: string, choice: ProductChoice) =>
		request<ItemSelection>(`/api/grocery-items/${itemId}/selection`, {
			method: "PUT",
			body: JSON.stringify(choice),
		}),
	/** Forgets the item's chosen product. */
	clear: (itemId: string) =>
		request<void>(`/api/grocery-items/${itemId}/selection`, {
			method: "DELETE",
		}),
};

/** The chosen products keyed by the item they were chosen for. */
export function selectionsByItem(
	selections: ItemSelection[],
): Map<string, ItemSelection> {
	return new Map(selections.map((s) => [s.grocery_item_id, s]));
}
