/**
 * How many list items a trolley handoff to a store would send.
 */
import type { GroceryItem, ItemSelection, StoreId } from "#/lib/api";

/**
 * Items still to buy (active or committed) whose chosen product is at
 * `store` — the same items the backend puts in that store's handoff.
 */
export function chosenAtStore(
	items: GroceryItem[],
	selections: Map<string, ItemSelection>,
	store: StoreId,
): number {
	return items.filter(
		(item) =>
			(item.status === "active" || item.status === "committed") &&
			selections.get(item.id)?.store === store,
	).length;
}
