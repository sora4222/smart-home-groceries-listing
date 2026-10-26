/**
 * How to undo a change to an item's chosen products.
 *
 * An item can have several products at a store, each with a count, and one
 * store is the one the order buys from. Undo puts back the changed store's
 * products and counts, then the store bought from. Pure: `lib/item-restore`
 * carries the plan out.
 */
import type { ItemSelection, StoreId } from "#/lib/api";

/** One product to choose again, and how many of it to buy. */
export interface CountedProduct {
	product_id: string;
	quantity: number;
}

/** What `store` held before the change, oldest choice first. */
export interface ChoiceUndo {
	store: StoreId;
	/** Empty when the store had no products. */
	products: CountedProduct[];
	/** Then make the order buy at this store again. */
	buyAt: StoreId | null;
}

/** Every product the item has at `store`, oldest first. */
export function productsAt(
	before: ItemSelection | null,
	store: StoreId,
): CountedProduct[] {
	if (!before) return [];
	const all = [
		before,
		...(before.bought_with ?? []),
		...(before.also_chosen ?? []),
	];
	return all
		.filter((s) => s.store === store)
		.map((s) => ({ product_id: s.product_id, quantity: s.quantity }));
}

/** The plan that puts `store` back as it was in `before`. */
export function undoAt(
	before: ItemSelection | null,
	store: StoreId,
): ChoiceUndo {
	const boughtAt = before?.store ?? null;
	return {
		store,
		products: productsAt(before, store),
		buyAt: boughtAt && boughtAt !== store ? boughtAt : null,
	};
}
