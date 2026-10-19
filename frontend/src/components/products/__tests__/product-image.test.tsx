import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ProductImage } from "#/components/products/product-image";

const MILK = "https://productimages.coles.com.au/productimages/8/8150288.jpg";

describe("ProductImage", () => {
	it("shows the store's picture named after the product", () => {
		render(<ProductImage src={MILK} name="Coles Full Cream Milk" />);

		const image = screen.getByRole("img", { name: "Coles Full Cream Milk" });
		expect(image).toHaveAttribute("src", MILK);
		expect(image).toHaveAttribute("referrerpolicy", "no-referrer");
	});

	it("shows a placeholder when the store has no picture", () => {
		render(<ProductImage src={null} name="Bananas" />);

		expect(
			screen.getByRole("img", { name: "No picture of Bananas" }),
		).toBeInTheDocument();
		expect(document.querySelector("img")).toBeNull();
	});

	it("swaps to the placeholder when the picture fails to load", () => {
		render(<ProductImage src={MILK} name="Milk" />);

		fireEvent.error(screen.getByRole("img", { name: "Milk" }));

		expect(
			screen.getByRole("img", { name: "No picture of Milk" }),
		).toBeInTheDocument();
	});

	it("tries again when the link changes after a failure", () => {
		const { rerender } = render(<ProductImage src={MILK} name="Milk" />);
		fireEvent.error(screen.getByRole("img", { name: "Milk" }));

		const other =
			"https://cdn0.woolworths.media/content/wowproductimages/small/007985.jpg";
		rerender(<ProductImage src={other} name="Milk" />);

		expect(screen.getByRole("img", { name: "Milk" })).toHaveAttribute(
			"src",
			other,
		);
	});
});
