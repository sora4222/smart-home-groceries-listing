import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { ProductRow } from "#/components/products/product-row";
import type { StoreProduct } from "#/lib/api";

function renderRow(product: StoreProduct, quantity = 1) {
	render(
		<ProductRow product={product} quantity={quantity} storeName="Coles" />,
	);
	return screen.getByRole("article", {
		name: [product.brand, product.name].filter(Boolean).join(" "),
	});
}

describe("ProductRow", () => {
	it("shows the product, its size, price and unit price", () => {
		const row = renderRow(productFixture());

		expect(row).toHaveTextContent("Coles Full Cream Milk");
		expect(row).toHaveTextContent("3L");
		expect(row).toHaveTextContent("$4.95");
		expect(row).toHaveTextContent("$0.165 / 100mL");
		expect(row).toHaveTextContent("unit price calculated from 1L");
	});

	it("strikes through the previous price of a special", () => {
		const row = renderRow(
			productFixture({ on_special: true, was_price: "6.2", price: "5.4" }),
		);

		expect(row).toHaveTextContent("Special");
		expect(row.querySelector("s")).toHaveTextContent("was $6.20");
	});

	it("shows a deal and what the item's quantity costs with it", () => {
		const row = renderRow(
			productFixture({
				price: "3.9",
				deals: [
					{ description: "2 for $6.50", min_quantity: 2, unit_price: "3.25" },
				],
				total_price: "6.5",
				deal_applied: true,
			}),
			2,
		);

		expect(row).toHaveTextContent("2 for $6.50");
		expect(row).toHaveTextContent("2 for $6.50 with deal");
	});

	it("says when the store gives no unit price", () => {
		const row = renderRow(
			productFixture({
				unit_price: null,
				unit_price_note: "no unit price from the store",
			}),
		);

		expect(row).toHaveTextContent("no unit price from the store");
	});

	it("marks an unavailable product and one with no price", () => {
		const row = renderRow(
			productFixture({ available: false, price: null, total_price: null }),
		);

		expect(row).toHaveTextContent("Unavailable");
		expect(row).toHaveTextContent("No price");
	});

	it("links to the product at the store in a new tab", () => {
		renderRow(productFixture());

		const link = screen.getByRole("link", {
			name: "View Coles Full Cream Milk at Coles (opens in a new tab)",
		});
		expect(link).toHaveAttribute("target", "_blank");
		expect(link).toHaveAttribute("rel", "noreferrer noopener");
	});
});
