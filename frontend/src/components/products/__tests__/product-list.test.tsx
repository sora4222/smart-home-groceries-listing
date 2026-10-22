import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { ProductList } from "#/components/products/product-list";
import type { StoreListing } from "#/lib/product-sort";

function listing(
	store: "coles" | "woolworths",
	name: string,
	productId = name,
): StoreListing {
	return {
		store,
		storeName: store === "coles" ? "Coles" : "Woolworths",
		product: productFixture({ product_id: productId, brand: null, name }),
	};
}

describe("ProductList", () => {
	it("lists both stores' products in the order given, each with its store", () => {
		render(
			<ProductList
				quantity={1}
				listings={[
					listing("woolworths", "Cheapest"),
					listing("coles", "Middle"),
					listing("woolworths", "Dearest"),
				]}
			/>,
		);

		const items = within(
			screen.getByRole("list", { name: "Products" }),
		).getAllByRole("listitem");
		expect(
			items.map((item) =>
				within(item).getByRole("article").getAttribute("aria-label"),
			),
		).toEqual(["Cheapest", "Middle", "Dearest"]);
		expect(items.map((item) => item.dataset.store)).toEqual([
			"woolworths",
			"coles",
			"woolworths",
		]);
		expect(items[1]).toHaveTextContent("Coles");
	});

	it("keeps the same product id at two stores apart", () => {
		render(
			<ProductList
				quantity={1}
				listings={[
					listing("coles", "Milk", "1"),
					listing("woolworths", "Milk", "1"),
				]}
			/>,
		);

		expect(screen.getAllByRole("article", { name: "Milk" })).toHaveLength(2);
	});

	it("shows nothing when there are no products", () => {
		render(<ProductList quantity={1} listings={[]} />);

		expect(screen.queryByRole("list")).toBeNull();
	});
});
