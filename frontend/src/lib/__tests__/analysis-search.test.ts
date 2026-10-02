import { describe, expect, it } from "vitest";

import { filtersFor, validateAnalysisSearch } from "#/lib/analysis-search";

describe("validateAnalysisSearch", () => {
	it("keeps what the page understands", () => {
		expect(
			validateAnalysisSearch({
				view: "item",
				range: "custom",
				from: "2026-09-01",
				to: "2026-09-30",
				store: "coles",
				item: "milk",
				category: "Bakery",
				period: "week",
			}),
		).toEqual({
			view: "item",
			range: "custom",
			from: "2026-09-01",
			to: "2026-09-30",
			store: "coles",
			item: "milk",
			category: "Bakery",
			period: "week",
		});
	});

	it("drops anything unknown or malformed", () => {
		expect(
			validateAnalysisSearch({
				view: "pie",
				range: "decade",
				from: "yesterday",
				store: "aldi",
				item: "   ",
				period: 7,
				extra: "x",
			}),
		).toEqual({});
	});
});

describe("filtersFor", () => {
	const today = new Date(2026, 9, 2);

	it("asks for all time, by month, when nothing is chosen", () => {
		expect(filtersFor({}, today)).toEqual({
			store: undefined,
			item: undefined,
			category: undefined,
			period: "month",
		});
	});

	it("turns a preset into days", () => {
		expect(filtersFor({ range: "month", store: "coles" }, today)).toMatchObject(
			{
				from: "2026-10-01",
				to: "2026-10-02",
				store: "coles",
			},
		);
	});

	it("uses the typed days only for a custom range", () => {
		const search = { from: "2026-01-01", to: "2026-01-31" };
		expect(filtersFor(search, today).from).toBeUndefined();
		expect(filtersFor({ ...search, range: "custom" }, today).from).toBe(
			"2026-01-01",
		);
	});
});
