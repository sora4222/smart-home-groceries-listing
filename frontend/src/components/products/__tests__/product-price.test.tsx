import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { ProductPrice } from "#/components/products/product-price";

describe("ProductPrice", () => {
	it('does not show a multiple of the price as "2 for" when no deal applies', () => {
		render(
			<ProductPrice
				product={productFixture({ price: "4.95", total_price: "9.90" })}
				quantity={2}
			/>,
		);

		expect(screen.getByText("$4.95")).toBeInTheDocument();
		expect(screen.queryByText(/2 for/)).not.toBeInTheDocument();
	});

	it("shows the total for several when a multibuy applies", () => {
		render(
			<ProductPrice
				product={productFixture({ total_price: "8.00", deal_applied: true })}
				quantity={2}
			/>,
		);

		expect(screen.getByText("2 for $8.00 with deal")).toBeInTheDocument();
	});

	it("hides the struck-through price when asked to", () => {
		const { container } = render(
			<ProductPrice
				product={productFixture({ was_price: "6.20", price: "5.40" })}
				quantity={1}
				showWasPrice={false}
			/>,
		);

		expect(container.querySelector("s")).toBeNull();
	});
});
