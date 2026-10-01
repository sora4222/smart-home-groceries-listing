import { createContext, useContext } from "react";

import type { ProductChoice } from "#/lib/api";

/**
 * What a price comparison knows about choosing: the product already chosen
 * for its item, and how to choose another.
 *
 * Scoped to one `<PriceComparison>` tree — never global. Outside one, the
 * value is `null` and the parts that choose render nothing, so product lists
 * can still be shown read-only.
 */
export interface ProductChoiceValue {
	/** The product chosen for the item, or `null` when there is none. */
	chosen: ProductChoice | null;
	/** Saves a new choice. Rejects when the backend refused it. */
	onChoose: (choice: ProductChoice) => Promise<void>;
}

const ProductChoiceContext = createContext<ProductChoiceValue | null>(null);

export const ProductChoiceProvider = ProductChoiceContext.Provider;

/** The comparison's choice, or `null` when it cannot choose. */
export function useProductChoice(): ProductChoiceValue | null {
	return useContext(ProductChoiceContext);
}

/** Whether `choice` names the same product at the same store as `chosen`. */
export function isChosen(
	chosen: ProductChoice | null,
	choice: ProductChoice,
): boolean {
	return (
		chosen !== null &&
		chosen.store === choice.store &&
		chosen.product_id === choice.product_id
	);
}
