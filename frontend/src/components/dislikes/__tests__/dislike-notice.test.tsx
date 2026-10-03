import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
	dislikeFixture,
	dislikesValue,
	renderWithDislikes,
} from "#/components/dislikes/__tests__/fixtures";
import { DislikeNotice } from "#/components/dislikes/dislike-notice";
import { productFixture } from "#/components/products/__tests__/fixtures";

const notice = <DislikeNotice store="coles" product={productFixture()} />;
const milk = { store: "coles", product_id: "c-milk-3l" } as const;

describe("DislikeNotice", () => {
	it("says who disliked the product", () => {
		renderWithDislikes(notice, dislikesValue({ dislikes: [dislikeFixture()] }));
		expect(screen.getByRole("status")).toHaveTextContent(
			"Phu disliked this item previously.",
		);
	});

	it("lets me buy it this time anyway", async () => {
		const value = dislikesValue({ dislikes: [dislikeFixture()] });
		renderWithDislikes(notice, value);

		await userEvent.click(
			screen.getByRole("button", { name: "Buy it this time" }),
		);

		expect(value.override).toHaveBeenCalledWith(milk);
	});

	it("shows an override for this item and can undo it", async () => {
		const value = dislikesValue({
			dislikes: [dislikeFixture()],
			overrides: [
				{
					grocery_item_id: "item-1",
					...milk,
					overridden_by: "dev-user",
					overridden_at: "2026-10-01T11:00:00Z",
				},
			],
		});
		renderWithDislikes(notice, value);

		expect(screen.getByText("OK to buy it this time.")).toBeInTheDocument();
		await userEvent.click(screen.getByRole("button", { name: "Undo" }));
		expect(value.clearOverride).toHaveBeenCalledWith(milk);
	});

	it("ignores an override on another item", () => {
		renderWithDislikes(
			notice,
			dislikesValue({
				dislikes: [dislikeFixture()],
				overrides: [
					{
						grocery_item_id: "item-2",
						store: "coles",
						product_id: "c-milk-3l",
						overridden_by: "dev-user",
						overridden_at: "2026-10-01T11:00:00Z",
					},
				],
			}),
		);
		expect(
			screen.getByRole("button", { name: "Buy it this time" }),
		).toBeInTheDocument();
	});

	it("renders nothing when nobody dislikes the product", () => {
		const { container } = renderWithDislikes(notice, dislikesValue());
		expect(container).toBeEmptyDOMElement();
	});

	it("renders nothing outside a price comparison", () => {
		const { container } = render(notice);
		expect(container).toBeEmptyDOMElement();
	});
});
