import { describe, expect, it } from "vitest";
import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import { undoAt } from "#/lib/choice-undo";

const atWoolworths = selectionFixture({
	store: "woolworths",
	store_name: "Woolworths",
	product_id: "w-milk-2l",
	quantity: 2,
});
const atColes = selectionFixture({
	store: "coles",
	product_id: "c-milk-3l",
	quantity: 1,
});

describe("undoAt", () => {
	it("leaves the store empty when the item had no choice", () => {
		expect(undoAt(null, "coles")).toEqual({
			store: "coles",
			products: [],
			buyAt: null,
		});
	});

	it("brings back the store's earlier product", () => {
		expect(undoAt(atColes, "coles")).toEqual({
			store: "coles",
			products: [{ product_id: "c-milk-3l", quantity: 1 }],
			buyAt: null,
		});
	});

	it("empties a new store and buys at the old store again", () => {
		expect(undoAt(atWoolworths, "coles")).toEqual({
			store: "coles",
			products: [],
			buyAt: "woolworths",
		});
	});

	it("brings back the other store's product and the store bought at", () => {
		const both = { ...atWoolworths, also_chosen: [atColes] };
		expect(undoAt(both, "coles")).toEqual({
			store: "coles",
			products: [{ product_id: "c-milk-3l", quantity: 1 }],
			buyAt: "woolworths",
		});
	});

	it("brings back every product at the store with its count, oldest first", () => {
		const second = selectionFixture({
			store: "woolworths",
			product_id: "w-milk-3l",
			quantity: 1,
		});
		const many = { ...atWoolworths, bought_with: [second] };
		expect(undoAt(many, "woolworths").products).toEqual([
			{ product_id: "w-milk-2l", quantity: 2 },
			{ product_id: "w-milk-3l", quantity: 1 },
		]);
	});
});
