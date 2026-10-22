import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { ChooseProductButton } from "#/components/products/choose-product-button";
import {
	ProductChoiceProvider,
	type ProductChoiceValue,
} from "#/components/products/product-choice";

function renderButton(
	choice: Partial<ProductChoiceValue> | null,
	product = productFixture(),
) {
	const button = (
		<ChooseProductButton store="coles" storeName="Coles" product={product} />
	);
	render(
		choice ? (
			<ProductChoiceProvider
				value={{ chosen: [], onChoose: vi.fn(), ...choice }}
			>
				{button}
			</ProductChoiceProvider>
		) : (
			button
		),
	);
}

describe("ChooseProductButton", () => {
	it("chooses the product at its store", async () => {
		const onChoose = vi.fn().mockResolvedValue(undefined);
		renderButton({ onChoose });

		await userEvent.click(
			screen.getByRole("button", {
				name: "Choose Coles Full Cream Milk at Coles",
			}),
		);

		expect(onChoose).toHaveBeenCalledWith({
			store: "coles",
			product_id: "c-milk-3l",
		});
	});

	it("marks the product already chosen", () => {
		renderButton({ chosen: [{ store: "coles", product_id: "c-milk-3l" }] });

		const button = screen.getByRole("button", {
			name: "Coles Full Cream Milk at Coles is chosen",
		});
		expect(button).toHaveAttribute("aria-pressed", "true");
		expect(button).toHaveTextContent("Chosen");
	});

	it("does not mark the same id at another store as chosen", () => {
		renderButton({
			chosen: [{ store: "woolworths", product_id: "c-milk-3l" }],
		});

		expect(screen.getByRole("button", { name: /^Choose / })).toHaveAttribute(
			"aria-pressed",
			"false",
		);
	});

	it("cannot choose an unavailable product", () => {
		renderButton({}, productFixture({ available: false }));

		expect(screen.getByRole("button", { name: /^Choose / })).toBeDisabled();
	});

	it("renders nothing outside a comparison that can choose", () => {
		renderButton(null);

		expect(screen.queryByRole("button")).not.toBeInTheDocument();
	});
});
