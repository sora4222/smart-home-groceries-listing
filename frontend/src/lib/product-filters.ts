/**
 * Narrowing the Select item list by store, specials, size measure and price.
 * Display filters only: they change what is shown, never what is bought.
 */
import type { StoreId, StoreProduct, StoreProducts } from "#/lib/api/products";
import type { StoreListing } from "#/lib/product-sort";
import { visibleProducts } from "#/lib/specials";

/** What a product is sold by. */
export type Measure = "grams" | "litres";

export interface ProductFilters {
	store: StoreId | "both";
	/** Only products on special. */
	specialsOnly: boolean;
	measure: Measure | "any";
	/** Highest shelf price to show, in dollars; `null` for no limit. */
	maxPrice: number | null;
}

export const NO_FILTERS: ProductFilters = {
	store: "both",
	specialsOnly: false,
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
 * Every product to show, as one list: from the stores the store filter
 * keeps that answered, on special when asked, in stock unless
 * `showUnavailable`, and passing the size and price filters.
 */
export function shownListings(
	stores: StoreProducts[],
	filters: ProductFilters,
	showUnavailable: boolean,
): StoreListing[] {
	const listings = storesShown(stores, filters)
		.filter((results) => results.status === "ok")
		.flatMap((results) =>
			visibleProducts(results.products, filters.specialsOnly)
				.filter((product) => showUnavailable || product.available)
				.map((product) => ({
					store: results.store,
					storeName: results.store_name,
					product,
				})),
		);
	return filterListings(listings, filters);
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
