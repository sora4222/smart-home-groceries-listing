/**
 * Sorting the price comparison's products, both stores mixed in one list.
 *
 * Every order puts unavailable products last, and products missing the
 * value sorted on (no price, no unit price, a unit price on another basis)
 * after those that have it, whichever direction is picked. Ties keep the
 * backend's order: cheapest per unit first within each store.
 */
import type { StoreId, StoreProduct } from "#/lib/api/products";

/** A product with the store it was found at, for one mixed list. */
export interface StoreListing {
	store: StoreId;
	storeName: string;
	product: StoreProduct;
}

export type ProductSort =
	| "unit-asc"
	| "unit-desc"
	| "price-asc"
	| "price-desc"
	| "name-asc"
	| "name-desc";

/** The sort choices, in the order the picker lists them. */
export const SORT_CHOICES: { value: ProductSort; label: string }[] = [
	{ value: "unit-asc", label: "Unit price: low to high" },
	{ value: "unit-desc", label: "Unit price: high to low" },
	{ value: "price-asc", label: "Price: low to high" },
	{ value: "price-desc", label: "Price: high to low" },
	{ value: "name-asc", label: "Name: A to Z" },
	{ value: "name-desc", label: "Name: Z to A" },
];

export const DEFAULT_SORT: ProductSort = "unit-asc";

/** The name a product is shown and sorted by: brand, then name. */
export function productTitle(product: StoreProduct): string {
	return [product.brand, product.name].filter(Boolean).join(" ");
}

/** A new list in the chosen order; the input is left as it was. */
export function sortListings(
	listings: StoreListing[],
	sort: ProductSort,
): StoreListing[] {
	const [field, direction] = sort.split("-") as [string, "asc" | "desc"];
	const sign = direction === "asc" ? 1 : -1;
	const readValue = valueReader(field, commonBasis(listings));

	return listings
		.map((listing, index) => ({ listing, index, value: readValue(listing) }))
		.sort(
			(a, b) =>
				Number(!a.listing.product.available) -
					Number(!b.listing.product.available) ||
				compareValues(a.value, b.value, sign) ||
				a.index - b.index,
		)
		.map(({ listing }) => listing);
}

type SortValue = number | string | null;

/** Reads the value one field sorts on; `null` when the product has none. */
function valueReader(
	field: string,
	basis: string | null,
): (listing: StoreListing) => SortValue {
	if (field === "name") return ({ product }) => productTitle(product);
	if (field === "price") return ({ product }) => toNumber(product.price);
	return ({ product }) =>
		product.unit_price && product.unit_price.per === basis
			? toNumber(product.unit_price.amount)
			: null;
}

/** Missing values last; otherwise by number or by name, in `sign`'s direction. */
function compareValues(a: SortValue, b: SortValue, sign: number): number {
	if (a === null || b === null) return Number(a === null) - Number(b === null);
	if (typeof a === "string" && typeof b === "string") {
		return sign * a.localeCompare(b, undefined, { sensitivity: "base" });
	}
	return sign * (Number(a) - Number(b));
}

/** The unit price basis most products share; others cannot be compared. */
function commonBasis(listings: StoreListing[]): string | null {
	const counts = new Map<string, number>();
	for (const { product } of listings) {
		const per = product.unit_price?.per;
		if (per) counts.set(per, (counts.get(per) ?? 0) + 1);
	}
	let best: string | null = null;
	for (const [per, count] of counts) {
		if (best === null || count > (counts.get(best) ?? 0)) best = per;
	}
	return best;
}

/** A decimal string as a number, or `null` when it is missing or unreadable. */
function toNumber(amount: string | null): number | null {
	if (amount === null) return null;
	const value = Number(amount);
	return Number.isFinite(value) ? value : null;
}
