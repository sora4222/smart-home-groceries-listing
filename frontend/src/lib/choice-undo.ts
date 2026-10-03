/**
 * How to undo a change to an item's chosen products.
 *
 * An item keeps one chosen product per store, and one of them is the product
 * the order buys. Choosing or clearing changes both, so Undo has to put back
 * the product at the store that changed *and* which store the order bought
 * from. Pure: the hook in `hooks/useProductChoices.ts` carries the plan out.
 */
import type { ItemSelection, ProductChoice, StoreId } from "#/lib/api";

/** The steps that put an item's choices back, in this order. */
export interface ChoiceUndo {
	/** Choose this product again (it becomes the one to buy). */
	choose: ProductChoice | null;
	/** Clear the product at this store (there was none before). */
	clear: StoreId | null;
	/** Then make the order buy at this store again. */
	buyAt: StoreId | null;
}

/** The product chosen at `store` before the change, if any. */
function previousAt(
	before: ItemSelection | null,
	store: StoreId,
): ProductChoice | null {
	if (!before) return null;
	const all = [before, ...(before.also_chosen ?? [])];
	const found = all.find((s) => s.store === store);
	return found ? { store: found.store, product_id: found.product_id } : null;
}

/**
 * Undoing a new choice at `store`: the store's earlier product comes back (or
 * the store is cleared), then the order buys where it did before.
 */
export function undoChoose(
	before: ItemSelection | null,
	store: StoreId,
): ChoiceUndo {
	const earlier = previousAt(before, store);
	const boughtAt = before?.store ?? null;
	return {
		choose: earlier,
		clear: earlier ? null : store,
		buyAt: boughtAt && boughtAt !== store ? boughtAt : null,
	};
}

/**
 * Undoing clearing the product at `store`: choose it again, then the order
 * buys where it did before.
 */
export function undoClear(
	before: ItemSelection | null,
	store: StoreId,
): ChoiceUndo {
	const cleared = previousAt(before, store);
	const boughtAt = before?.store ?? null;
	return {
		choose: cleared,
		clear: null,
		buyAt: boughtAt && boughtAt !== store ? boughtAt : null,
	};
}
