import type { GroceryItem } from "#/lib/api";

/**
 * A list item with the fields a given test does not care about already filled
 * in. Shared by the grocery component tests so each one states only what it is
 * actually about.
 */
export function itemFixture(overrides: Partial<GroceryItem> = {}): GroceryItem {
	return {
		id: "33333333-3333-3333-3333-333333333333",
		name: "milk",
		quantity: 2,
		status: "active",
		source: "manual",
		note: null,
		filter_terms: [],
		added_by_user_id: "user_dev",
		created_at: new Date("2026-09-30T00:00:00Z").toISOString(),
		...overrides,
	};
}
