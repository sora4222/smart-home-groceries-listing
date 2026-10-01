import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
	productFixture,
	storeFixture,
} from "#/components/products/__tests__/fixtures";
import { StoreResults } from "#/components/products/store-results";
import type { StoreProducts } from "#/lib/api";

function renderStore(results: StoreProducts, specialsOnly = false) {
	render(
		<StoreResults results={results} quantity={1} specialsOnly={specialsOnly} />,
	);
	return screen.getByRole("region", { name: results.store_name });
}

describe("StoreResults", () => {
	it("lists the store's products in the order given", () => {
		const region = renderStore(
			storeFixture({
				products: [
					productFixture({ product_id: "a", name: "Cheapest" }),
					productFixture({ product_id: "b", name: "Dearer" }),
				],
			}),
		);

		const names = within(region)
			.getAllByRole("article")
			.map((a) => a.getAttribute("aria-label"));
		expect(names).toEqual(["Coles Cheapest", "Coles Dearer"]);
	});

	it("says why a store that failed shows nothing", () => {
		const region = renderStore(
			storeFixture({
				status: "unreachable",
				message: "Coles could not be reached. Try again later.",
				products: [],
			}),
		);

		expect(within(region).getByRole("status")).toHaveTextContent(
			"Coles could not be reached. Try again later.",
		);
	});

	it("says when nothing matches the item and its filters", () => {
		const region = renderStore(storeFixture({ products: [] }));

		expect(region).toHaveTextContent(
			"No products at Coles match this item and its filters.",
		);
	});

	it("shows only promoted products when specials only is on", () => {
		const region = renderStore(
			storeFixture({
				products: [
					productFixture({ product_id: "a", name: "Plain" }),
					productFixture({
						product_id: "b",
						name: "On special",
						on_special: true,
					}),
					productFixture({
						product_id: "c",
						name: "Multibuy",
						deals: [
							{ description: "2 for $5", min_quantity: 2, unit_price: "2.5" },
						],
					}),
				],
			}),
			true,
		);

		const names = within(region)
			.getAllByRole("article")
			.map((a) => a.getAttribute("aria-label"));
		expect(names).toEqual(["Coles On special", "Coles Multibuy"]);
	});

	it("says when nothing is on special", () => {
		const region = renderStore(storeFixture(), true);

		expect(region).toHaveTextContent("Nothing on special at Coles.");
	});
});
