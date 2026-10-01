import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { lineFixture } from "#/components/order/__tests__/fixtures";
import { OrderLineRow } from "#/components/order/order-line-row";

function renderLine(overrides = {}) {
	render(
		<ul>
			<OrderLineRow line={lineFixture(overrides)} />
		</ul>,
	);
	return screen.getByRole("listitem", { name: "milk" });
}

describe("OrderLineRow", () => {
	it("shows the item, its quantity, the product and today's total", () => {
		const row = renderLine();

		expect(row).toHaveTextContent("milk");
		expect(row).toHaveTextContent("× 2");
		expect(row).toHaveTextContent("Coles Full Cream Milk");
		expect(row).toHaveTextContent("3L");
		expect(row).toHaveTextContent("$9.90");
		expect(row).toHaveTextContent("$4.95 each");
	});

	it("links the product to its page at the store", () => {
		renderLine();

		const link = screen.getByRole("link", { name: /Full Cream Milk/ });
		expect(link).toHaveAttribute(
			"href",
			"https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288",
		);
		expect(link).toHaveAttribute("target", "_blank");
	});

	it("flags a price that went up since it was chosen", () => {
		const row = renderLine({ price_change: "up", chosen_price: "4.50" });

		expect(row).toHaveTextContent("Up from $4.50");
	});

	it("says when a deal lowers the total", () => {
		const row = renderLine({ deal_applied: true });

		expect(row).toHaveTextContent("Deal applied");
	});

	it("shows the problem, and no price, when the line cannot be bought", () => {
		const row = renderLine({
			status: "not_offered",
			price: null,
			total_price: null,
			price_change: null,
			problem:
				"Coles no longer offers this product for milk — choose another on the list.",
		});

		expect(row).toHaveTextContent("No price");
		expect(row).not.toHaveTextContent("$9.90");
		expect(row).toHaveTextContent("no longer offers this product");
	});
});
