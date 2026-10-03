import { createContext, useContext } from "react";

import type { ProductChoice } from "#/lib/api";

/**
 * What a price comparison knows about choosing: the products already chosen
 * for its item (one per store at most), and how to choose another.
 *
 * Scoped to one `<PriceComparison>` tree — never global. Outside one, the
 * value is `null` and the parts that choose render nothing, so product lists
 * can still be shown read-only.
 */
export interface ProductChoiceValue {
	/** The products chosen for the item, at any store. */
	chosen: ProductChoice[];
	/** Saves a new choice. Rejects when the backend refused it. */
	onChoose: (choice: ProductChoice) => Promise<void>;
}

const ProductChoiceContext = createContext<ProductChoiceValue | null>(null);

export const ProductChoiceProvider = ProductChoiceContext.Provider;

/** The comparison's choice, or `null` when it cannot choose. */
export function useProductChoice(): ProductChoiceValue | null {
	return useContext(ProductChoiceContext);
}

/** Whether `choice` names a product, at its store, that is in `chosen`. */
export function isChosen(
	chosen: ProductChoice[],
	choice: ProductChoice,
): boolean {
	return chosen.some(
		(c) => c.store === choice.store && c.product_id === choice.product_id,
	);
}
