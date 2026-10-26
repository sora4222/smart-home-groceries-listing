import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import type { StoreId, StoreProduct, StoreProducts } from "#/lib/api";
import {
	filterListings,
	measureOf,
	NO_FILTERS,
	storesShown,
} from "#/lib/product-filters";
import type { StoreListing } from "#/lib/product-sort";

function listing(
	store: "woolworths" | "coles",
	overrides: Partial<StoreProduct>,
): StoreListing {
	return {
		store,
		storeName: store === "coles" ? "Coles" : "Woolworths",
		product: productFixture(overrides),
	};
}

const milk = listing("coles", { product_id: "milk", price: "4.95" });
const rice = listing("woolworths", {
	product_id: "rice",
	price: "8.00",
	unit_price: { amount: "0.80", per: "100g", converted_from: null },
	package_size: "1kg",
});
const eggs = listing("woolworths", {
	product_id: "eggs",
	price: null,
	unit_price: { amount: "0.50", per: "unit", converted_from: null },
	package_size: "12 pack",
});

const ids = (listings: StoreListing[]) =>
	listings.map((l) => l.product.product_id);

describe("measureOf", () => {
	it("reads the unit price basis", () => {
		expect(measureOf(milk.product)).toBe("litres");
		expect(measureOf(rice.product)).toBe("grams");
		expect(measureOf(eggs.product)).toBeNull();
	});

	it("falls back to the package size", () => {
		expect(
			measureOf(productFixture({ unit_price: null, package_size: "500 g" })),
		).toBe("grams");
		expect(
			measureOf(productFixture({ unit_price: null, package_size: "1.5L" })),
		).toBe("litres");
		expect(
			measureOf(productFixture({ unit_price: null, package_size: null })),
		).toBeNull();
	});
});

describe("filterListings", () => {
	const all = [milk, rice, eggs];

	it("keeps everything with no filters", () => {
		expect(ids(filterListings(all, NO_FILTERS))).toEqual([
			"milk",
			"rice",
			"eggs",
		]);
	});

	it("keeps one store's results", () => {
		const results = (store: StoreId): StoreProducts => ({
			store,
			store_name: store,
			status: "ok",
			message: null,
			products: [],
		});
		const both = [results("woolworths"), results("coles")];
		expect(
			storesShown(both, { ...NO_FILTERS, store: "coles" }).map((s) => s.store),
		).toEqual(["coles"]);
		expect(storesShown(both, NO_FILTERS)).toHaveLength(2);
	});

	it("keeps grams or litres", () => {
		expect(
			ids(filterListings(all, { ...NO_FILTERS, measure: "grams" })),
		).toEqual(["rice"]);
		expect(
			ids(filterListings(all, { ...NO_FILTERS, measure: "litres" })),
		).toEqual(["milk"]);
	});

	it("drops products over the price, and those with no price", () => {
		expect(ids(filterListings(all, { ...NO_FILTERS, maxPrice: 5 }))).toEqual([
			"milk",
		]);
	});
});
