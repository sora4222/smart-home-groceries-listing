import { describe, expect, it } from "vitest";

import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import type { GroceryItem, GroceryItemStatus } from "#/lib/api";
import { chosenAtStore } from "#/lib/chosen-at-store";

function item(id: string, status: GroceryItemStatus): GroceryItem {
	return {
		id,
		name: id,
		quantity: 1,
		status,
		source: "manual",
		note: null,
		filter_terms: [],
		added_by_user_id: null,
		created_at: "2026-10-01T00:00:00Z",
	};
}

describe("chosenAtStore", () => {
	const selections = new Map([
		["a", selectionFixture({ grocery_item_id: "a", store: "woolworths" })],
		["b", selectionFixture({ grocery_item_id: "b", store: "woolworths" })],
		["c", selectionFixture({ grocery_item_id: "c", store: "coles" })],
		["d", selectionFixture({ grocery_item_id: "d", store: "woolworths" })],
	]);

	it("counts active and committed items chosen at the store", () => {
		const items = [
			item("a", "active"),
			item("b", "committed"),
			item("c", "committed"),
			item("e", "committed"),
		];

		expect(chosenAtStore(items, selections, "woolworths")).toBe(2);
		expect(chosenAtStore(items, selections, "coles")).toBe(1);
	});

	it("leaves out items already ordered", () => {
		expect(
			chosenAtStore([item("d", "ordered")], selections, "woolworths"),
		).toBe(0);
	});
});
