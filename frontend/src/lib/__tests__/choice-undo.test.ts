import { describe, expect, it } from "vitest";
import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import { undoChoose, undoClear } from "#/lib/choice-undo";

const atWoolworths = selectionFixture({
	store: "woolworths",
	store_name: "Woolworths",
	product_id: "w-milk-2l",
});
const atColes = selectionFixture({ store: "coles", product_id: "c-milk-3l" });

describe("undoChoose", () => {
	it("clears the store when the item had no choice at all", () => {
		expect(undoChoose(null, "coles")).toEqual({
			choose: null,
			clear: "coles",
			buyAt: null,
		});
	});

	it("brings back the store's earlier product", () => {
		expect(undoChoose(atColes, "coles")).toEqual({
			choose: { store: "coles", product_id: "c-milk-3l" },
			clear: null,
			buyAt: null,
		});
	});

	it("clears a new store and buys at the old store again", () => {
		expect(undoChoose(atWoolworths, "coles")).toEqual({
			choose: null,
			clear: "coles",
			buyAt: "woolworths",
		});
	});

	it("brings back the other store's earlier product and the store bought at", () => {
		const both = { ...atWoolworths, also_chosen: [atColes] };
		expect(undoChoose(both, "coles")).toEqual({
			choose: { store: "coles", product_id: "c-milk-3l" },
			clear: null,
			buyAt: "woolworths",
		});
	});
});

describe("undoClear", () => {
	it("chooses the cleared product again", () => {
		expect(undoClear(atColes, "coles")).toEqual({
			choose: { store: "coles", product_id: "c-milk-3l" },
			clear: null,
			buyAt: null,
		});
	});

	it("keeps buying at the store the order used when the other store was cleared", () => {
		const both = { ...atWoolworths, also_chosen: [atColes] };
		expect(undoClear(both, "coles")).toEqual({
			choose: { store: "coles", product_id: "c-milk-3l" },
			clear: null,
			buyAt: "woolworths",
		});
	});
});
