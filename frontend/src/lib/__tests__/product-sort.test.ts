import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import type { StoreProduct } from "#/lib/api";
import {
	type ProductSort,
	type StoreListing,
	sortListings,
} from "#/lib/product-sort";

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

function namesAfter(listings: StoreListing[], sort: ProductSort): string[] {
	return sortListings(listings, sort).map(({ product }) => product.name);
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
			sortListings(branded, "name-asc").map(({ product }) => product.brand),
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
});
