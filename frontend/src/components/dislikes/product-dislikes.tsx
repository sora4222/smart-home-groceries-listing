import { createContext, useContext } from "react";

import type {
	DislikedProduct,
	DislikeOverride,
	ProductChoice,
	ProductDislike,
} from "#/lib/api";

/**
 * What one price comparison knows about dislikes: everyone's dislikes, the
 * overrides, the item being compared, and how to change them.
 *
 * Scoped to one `<PriceComparison>` tree — never global. Outside one the
 * value is `null` and the dislike parts render nothing.
 */
export interface ProductDislikesValue {
	itemId: string;
	dislikes: ProductDislike[];
	overrides: DislikeOverride[];
	dislike: (product: DislikedProduct) => Promise<void>;
	removeMine: (choice: ProductChoice) => Promise<void>;
	override: (choice: ProductChoice) => Promise<void>;
	clearOverride: (choice: ProductChoice) => Promise<void>;
}

const ProductDislikesContext = createContext<ProductDislikesValue | null>(null);

export const ProductDislikesProvider = ProductDislikesContext.Provider;

/** The comparison's dislikes, or `null` outside one. */
export function useProductDislikes(): ProductDislikesValue | null {
	return useContext(ProductDislikesContext);
}
