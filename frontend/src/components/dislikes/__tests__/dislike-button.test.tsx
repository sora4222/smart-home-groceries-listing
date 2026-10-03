import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
	dislikeFixture,
	dislikesValue,
	renderWithDislikes,
} from "#/components/dislikes/__tests__/fixtures";
import { DislikeButton } from "#/components/dislikes/dislike-button";
import { productFixture } from "#/components/products/__tests__/fixtures";

const button = (
	<DislikeButton store="coles" storeName="Coles" product={productFixture()} />
);

describe("DislikeButton", () => {
	it("dislikes the product with the label the comparison shows", async () => {
		const value = dislikesValue();
		renderWithDislikes(button, value);

		await userEvent.click(
			screen.getByRole("button", {
				name: "Dislike Coles Full Cream Milk at Coles",
			}),
		);

		expect(value.dislike).toHaveBeenCalledWith({
			store: "coles",
			product_id: "c-milk-3l",
			name: "Full Cream Milk",
			brand: "Coles",
			package_size: "3L",
		});
	});

	it("removes my own dislike when I already dislike it", async () => {
		const value = dislikesValue({ dislikes: [dislikeFixture({ mine: true })] });
		renderWithDislikes(button, value);

		await userEvent.click(
			screen.getByRole("button", {
				name: "Remove my dislike of Coles Full Cream Milk at Coles",
			}),
		);

		expect(value.removeMine).toHaveBeenCalledWith({
			store: "coles",
			product_id: "c-milk-3l",
		});
	});

	it("still offers Dislike when only someone else dislikes it", () => {
		renderWithDislikes(button, dislikesValue({ dislikes: [dislikeFixture()] }));
		expect(
			screen.getByRole("button", {
				name: "Dislike Coles Full Cream Milk at Coles",
			}),
		).toBeInTheDocument();
	});

	it("renders nothing outside a price comparison", () => {
		const { container } = render(button);
		expect(container).toBeEmptyDOMElement();
	});
});
