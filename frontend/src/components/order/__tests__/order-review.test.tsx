import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
	lineFixture,
	reviewFixture,
	storeFixture,
} from "#/components/order/__tests__/fixtures";
import { OrderReview } from "#/components/order/order-review";
import type { OrderReview as OrderReviewData } from "#/lib/api";

function renderReview(review: OrderReviewData) {
	render(
		<OrderReview review={review}>
			<OrderReview.Empty>
				<a href="/">Back to the list</a>
			</OrderReview.Empty>
			<OrderReview.Unchosen>
				<a href="/">Choose products</a>
			</OrderReview.Unchosen>
			<OrderReview.Store store="woolworths">
				<button type="button">Send to Woolworths</button>
			</OrderReview.Store>
			<OrderReview.Store store="coles" />
			<OrderReview.Total />
		</OrderReview>,
	);
}

const empty: OrderReviewData = {
	stores: [],
	unchosen: [],
	total: "0",
	complete: true,
};

describe("OrderReview", () => {
	it("explains how to start an order when nothing is committed", () => {
		renderReview(empty);

		expect(screen.getByText(/Nothing is ready to order yet/)).toBeVisible();
		expect(
			screen.getByRole("link", { name: "Back to the list" }),
		).toBeVisible();
		expect(screen.queryByText(/Total/)).not.toBeInTheDocument();
	});

	it("shows each store's items and subtotal, with that store's actions", () => {
		renderReview(reviewFixture());

		const woolworths = screen.getByRole("region", { name: "Woolworths" });
		expect(within(woolworths).getByText("bananas")).toBeVisible();
		expect(woolworths).toHaveTextContent("Subtotal$4.00");
		expect(
			within(woolworths).getByRole("button", { name: "Send to Woolworths" }),
		).toBeVisible();

		const coles = screen.getByRole("region", { name: "Coles" });
		expect(within(coles).getByText("milk")).toBeVisible();
		expect(coles).toHaveTextContent("Subtotal$9.90");
		expect(screen.queryByText(/Nothing is ready to order/)).toBeNull();
	});

	it("leaves out a store with nothing to buy", () => {
		renderReview(reviewFixture({ stores: [storeFixture()] }));

		expect(screen.queryByRole("region", { name: "Woolworths" })).toBeNull();
		expect(screen.queryByRole("button", { name: "Send to Woolworths" })).toBe(
			null,
		);
	});

	it("shows the total for items and says delivery is not included", () => {
		renderReview(reviewFixture());

		const total = screen.getByRole("region", { name: "Order total" });
		expect(total).toHaveTextContent("Total for items$13.90");
		expect(total).toHaveTextContent("Delivery fees are not included");
	});

	it("lists items with no product and sends the household to choose one", () => {
		renderReview(
			reviewFixture({
				unchosen: [{ grocery_item_id: "b", name: "bread", quantity: 2 }],
				complete: false,
			}),
		);

		const unchosen = screen.getByRole("region", {
			name: "Items with no product",
		});
		expect(unchosen).toHaveTextContent("bread");
		expect(unchosen).toHaveTextContent("× 2");
		expect(
			within(unchosen).getByRole("link", { name: "Choose products" }),
		).toBeVisible();
		expect(
			screen.getByRole("region", { name: "Order total" }),
		).toHaveTextContent("Total so far");
	});

	it("marks a store's subtotal as so far when a line has no price", () => {
		renderReview(
			reviewFixture({
				stores: [
					storeFixture({
						complete: false,
						lines: [
							lineFixture(),
							lineFixture({
								grocery_item_id: "x",
								item_name: "eggs",
								status: "store_failed",
								total_price: null,
								price: null,
								problem: "Coles could not be reached. Try again later.",
							}),
						],
					}),
				],
				complete: false,
			}),
		);

		const coles = screen.getByRole("region", { name: "Coles" });
		expect(coles).toHaveTextContent("Subtotal so far$9.90");
		expect(coles).toHaveTextContent("Coles could not be reached");
	});
});
