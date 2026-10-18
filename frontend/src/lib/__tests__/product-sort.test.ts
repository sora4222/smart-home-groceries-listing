import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import type { StoreProduct } from "#/lib/api";
import {
	commonBasis,
	type ProductSort,
	type SortContext,
	type StoreListing,
	sortListings,
} from "#/lib/product-sort";

const context: SortContext = { quantity: 1, basis: "100mL" };

function listing(
	product: Partial<StoreProduct>,
	store: "coles" | "woolworths" = "coles",
): StoreListing {
	return {
		store,
		storeName: store === "coles" ? "Coles" : "Woolworths",
		product: productFixture({ brand: null, ...product }),
	};
}

function unit(amount: string, per: "100mL" | "100g" | "unit" = "100mL") {
	return { amount, per, converted_from: null };
}

function namesAfter(
	listings: StoreListing[],
	sort: ProductSort,
	given: SortContext = context,
): string[] {
	return sortListings(listings, sort, given).map(({ product }) => product.name);
}

const mixed = [
	listing({
		product_id: "1",
		name: "Bravo",
		price: "4.00",
		unit_price: unit("0.30"),
	}),
	listing(
		{ product_id: "2", name: "alpha", price: "2.50", unit_price: unit("0.40") },
		"woolworths",
	),
	listing({
		product_id: "3",
		name: "Charlie",
		price: "6.00",
		unit_price: unit("0.10"),
	}),
];

describe("sortListings", () => {
	it("sorts by unit price both ways, mixing the stores", () => {
		expect(namesAfter(mixed, "unit-asc")).toEqual([
			"Charlie",
			"Bravo",
			"alpha",
		]);
		expect(namesAfter(mixed, "unit-desc")).toEqual([
			"alpha",
			"Bravo",
			"Charlie",
		]);
	});

	it("sorts by price both ways", () => {
		expect(namesAfter(mixed, "price-asc")).toEqual([
			"alpha",
			"Bravo",
			"Charlie",
		]);
		expect(namesAfter(mixed, "price-desc")).toEqual([
			"Charlie",
			"Bravo",
			"alpha",
		]);
	});

	it("sorts by name both ways, ignoring case", () => {
		expect(namesAfter(mixed, "name-asc")).toEqual([
			"alpha",
			"Bravo",
			"Charlie",
		]);
		expect(namesAfter(mixed, "name-desc")).toEqual([
			"Charlie",
			"Bravo",
			"alpha",
		]);
	});

	it("sorts by brand and name together", () => {
		const branded = [
			listing({ product_id: "1", brand: "Zed", name: "Milk" }),
			listing({ product_id: "2", brand: "Acme", name: "Milk" }),
		];

		expect(
			sortListings(branded, "name-asc", context).map(
				({ product }) => product.brand,
			),
		).toEqual(["Acme", "Zed"]);
	});

	it("puts products missing the value last in either direction", () => {
		const listings = [
			listing({
				product_id: "1",
				name: "No price",
				price: null,
				unit_price: null,
			}),
			...mixed,
		];

		expect(namesAfter(listings, "price-desc").at(-1)).toBe("No price");
		expect(namesAfter(listings, "price-asc").at(-1)).toBe("No price");
		expect(namesAfter(listings, "unit-desc").at(-1)).toBe("No price");
	});

	it("puts unit prices on another basis after the comparable ones", () => {
		const listings = [
			listing({
				product_id: "1",
				name: "Each",
				unit_price: unit("0.01", "unit"),
			}),
			...mixed,
		];

		expect(namesAfter(listings, "unit-asc").at(-1)).toBe("Each");
	});

	it("puts unavailable products last", () => {
		const listings = [
			listing({ product_id: "1", name: "Aardvark", available: false }),
			...mixed,
		];

		expect(namesAfter(listings, "name-asc").at(-1)).toBe("Aardvark");
	});

	it("keeps the given order for ties and leaves the input alone", () => {
		const ties = [
			listing({ product_id: "1", name: "First" }),
			listing({ product_id: "2", name: "Second" }, "woolworths"),
		];

		expect(namesAfter(ties, "price-asc")).toEqual(["First", "Second"]);
		expect(namesAfter(ties, "price-desc")).toEqual(["First", "Second"]);
		expect(ties[0].product.name).toBe("First");
	});

	it("judges unit price after a multibuy at the item's quantity", () => {
		const listings = [
			listing({
				product_id: "1",
				name: "Plain",
				price: "3.00",
				unit_price: unit("0.30"),
				total_price: "6.00",
			}),
			listing({
				product_id: "2",
				name: "Multibuy",
				price: "3.50",
				unit_price: unit("0.35"),
				// 2 for $5: 25c per 100mL once two are bought.
				total_price: "5.00",
				deal_applied: true,
			}),
		];

		expect(
			namesAfter(listings, "unit-asc", { quantity: 2, basis: "100mL" }),
		).toEqual(["Multibuy", "Plain"]);
	});

	it("compares on the basis it is given, not the one left after a filter", () => {
		const listings = [
			listing({
				product_id: "1",
				name: "Each",
				unit_price: unit("0.01", "unit"),
			}),
			...mixed,
		];

		expect(namesAfter(listings, "unit-asc").at(-1)).toBe("Each");
		expect(
			namesAfter(listings, "unit-asc", { quantity: 1, basis: "unit" })[0],
		).toBe("Each");
	});
});

describe("commonBasis", () => {
	it("is the basis most unit prices share", () => {
		expect(
			commonBasis(
				[unit("1", "unit"), unit("1"), unit("2"), null].map((unit_price) =>
					productFixture({ unit_price }),
				),
			),
		).toBe("100mL");
	});

	it("is null when nothing has a unit price", () => {
		expect(commonBasis([productFixture({ unit_price: null })])).toBeNull();
	});
});
