import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
	PastPurchases,
	PastPurchasesProvider,
} from "#/components/products/past-purchases";
import type { ProductHistory } from "#/lib/api";

const milk: ProductHistory = {
	store: "coles",
	product_id: "c-milk-3l",
	times_bought: 2,
	purchases: [
		{
			bought_at: "2026-09-20T02:00:00Z",
			quantity: 2,
			unit_price: "4.95",
			total_price: "9.90",
		},
		{
			bought_at: "2026-09-01T02:00:00Z",
			quantity: 1,
			unit_price: "4.50",
			total_price: "4.50",
		},
	],
};

function renderFor(productId: string) {
	render(
		<PastPurchasesProvider history={new Map([["coles:c-milk-3l", milk]])}>
			<PastPurchases store="coles" productId={productId} productName="Milk" />
		</PastPurchasesProvider>,
	);
}

describe("PastPurchases", () => {
	it("says how often a product was bought and opens to the prices", async () => {
		renderFor("c-milk-3l");
		const button = screen.getByRole("button", { name: /Bought Milk 2 times/ });
		expect(button).toHaveTextContent("Bought 2 times");

		await userEvent.click(button);

		const rows = screen.getAllByRole("listitem");
		expect(rows[0]).toHaveTextContent("20 Sept 2026");
		expect(rows[0]).toHaveTextContent("$4.95 each × 2");
		expect(rows[1]).toHaveTextContent("$4.50 each × 1");
	});

	it("shows nothing for a product never bought", () => {
		renderFor("c-oat-1l");
		expect(screen.queryByRole("button")).toBeNull();
	});
});
