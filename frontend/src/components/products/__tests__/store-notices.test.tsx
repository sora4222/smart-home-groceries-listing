import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
	productFixture,
	storeFixture,
} from "#/components/products/__tests__/fixtures";
import { StoreNotices } from "#/components/products/store-notices";
import type { StoreProducts } from "#/lib/api";

function renderNotices(stores: StoreProducts[], specialsOnly = false) {
	render(<StoreNotices stores={stores} specialsOnly={specialsOnly} />);
}

describe("StoreNotices", () => {
	it("says why a store that failed shows nothing", () => {
		renderNotices([
			storeFixture({
				status: "unreachable",
				message: "Coles could not be reached. Try again later.",
				products: [],
			}),
		]);

		expect(screen.getByRole("status")).toHaveTextContent(
			"Coles could not be reached. Try again later.",
		);
	});

	it("says when nothing matches the item and its filters", () => {
		renderNotices([storeFixture({ products: [] })]);

		expect(screen.getByRole("status")).toHaveTextContent(
			"No products at Coles match this item and its filters.",
		);
	});

	it("says when nothing is on special", () => {
		renderNotices([storeFixture()], true);

		expect(screen.getByRole("status")).toHaveTextContent(
			"Nothing on special at Coles.",
		);
	});

	it("says nothing for a store with products to show", () => {
		renderNotices(
			[storeFixture({ products: [productFixture({ on_special: true })] })],
			true,
		);

		expect(screen.queryByRole("status")).toBeNull();
	});
});
