/**
 * The "specials only" display filter on the price comparison.
 *
 * It narrows what is shown and nothing else: the order stays cheapest per
 * unit first, and the later order optimiser ignores it unless the user
 * turns on "prefer specials" in their order settings.
 */
import type { StoreProduct } from "#/lib/api/products";

/** A product is on promotion: marked special, or carrying a multibuy. */
export function isPromoted(product: StoreProduct): boolean {
	return product.on_special || product.deals.length > 0;
}

/** The products to show, given whether only specials are wanted. */
export function visibleProducts(
	products: StoreProduct[],
	specialsOnly: boolean,
): StoreProduct[] {
	return specialsOnly ? products.filter(isPromoted) : products;
}
