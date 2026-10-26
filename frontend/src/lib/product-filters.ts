/**
 * Narrowing the Select item list by store, size measure and price. Display
 * filters only: they change what is shown, never what is bought.
 */
import type { StoreId, StoreProduct, StoreProducts } from "#/lib/api/products";
import type { StoreListing } from "#/lib/product-sort";

/** What a product is sold by. */
export type Measure = "grams" | "litres";

export interface ProductFilters {
	store: StoreId | "both";
	measure: Measure | "any";
	/** Highest shelf price to show, in dollars; `null` for no limit. */
	maxPrice: number | null;
}

export const NO_FILTERS: ProductFilters = {
	store: "both",
	measure: "any",
	maxPrice: null,
};

const SIZE = /(\d+(?:\.\d+)?)\s*(kg|g|ml|l)\b/i;

/**
 * Whether a product is sold by weight or volume: from its unit price basis,
 * or else from its package size ("2L", "500g"). `null` when neither says.
 */
export function measureOf(product: StoreProduct): Measure | null {
	const per = product.unit_price?.per;
	if (per === "100g") return "grams";
	if (per === "100mL") return "litres";
	const unit = product.package_size?.match(SIZE)?.[2].toLowerCase();
	if (unit === "g" || unit === "kg") return "grams";
	if (unit === "ml" || unit === "l") return "litres";
	return null;
}

/** The stores the store filter keeps, for their products and notices. */
export function storesShown(
	stores: StoreProducts[],
	filters: ProductFilters,
): StoreProducts[] {
	return stores.filter(
		(results) => filters.store === "both" || results.store === filters.store,
	);
}

/**
 * The listings that pass the size and price filters, in the order given.
 * The store filter is applied before, by `storesShown`.
 */
export function filterListings(
	listings: StoreListing[],
	filters: ProductFilters,
): StoreListing[] {
	return listings.filter(
		({ product }) =>
			(filters.measure === "any" || measureOf(product) === filters.measure) &&
			withinPrice(product, filters.maxPrice),
	);
}

/** A product with no price is kept only when there is no limit. */
function withinPrice(product: StoreProduct, maxPrice: number | null): boolean {
	if (maxPrice === null) return true;
	const price = Number(product.price);
	return product.price !== null && Number.isFinite(price) && price <= maxPrice;
}
