import { describe, expect, it } from "vitest";
import type { OptionStore, OrderOption } from "#/lib/api";
import { deliveryPhrase, optionNotes } from "#/lib/option-notes";

function store(overrides: Partial<OptionStore> = {}): OptionStore {
	return {
		store: "woolworths",
		store_name: "Woolworths",
		lines: [],
		subtotal: "40",
		delivery_fee: "9",
		fee_known: true,
		free_delivery: false,
		below_minimum: false,
		minimum_order: null,
		...overrides,
	};
}

function option(overrides: Partial<OrderOption> = {}): OrderOption {
	return {
		kind: "woolworths",
		label: "All at Woolworths",
		recommended: true,
		is_current: false,
		stores: [store()],
		missing: [],
		items_total: "40",
		delivery_total: "9",
		total: "49",
		complete: true,
		fees_known: true,
		meets_minimums: true,
		within_delivery_cap: true,
		picks: [],
		...overrides,
	};
}

describe("deliveryPhrase", () => {
	it("says free, not set, or the fee", () => {
		expect(deliveryPhrase(store({ free_delivery: true }))).toBe(
			"free delivery",
		);
		expect(deliveryPhrase(store({ fee_known: false }))).toBe(
			"delivery fee not set",
		);
		expect(deliveryPhrase(store())).toBe("$9.00 delivery");
	});
});

describe("optionNotes", () => {
	it("has nothing to say about a clean option", () => {
		expect(optionNotes(option(), null)).toEqual([]);
	});

	it("lists every problem, missing items first", () => {
		const notes = optionNotes(
			option({
				complete: false,
				missing: [{ grocery_item_id: "1", name: "bread", reason: "x" }],
				stores: [store({ below_minimum: true, minimum_order: "50" })],
				within_delivery_cap: false,
				fees_known: false,
			}),
			"5",
		);
		expect(notes).toEqual([
			"Leaves out 1 item.",
			"Under the Woolworths minimum order of $50.00.",
			"Delivery is over your $5.00 limit.",
			"Some delivery fees are not set, so they count as $0.",
		]);
	});
});
