import { describe, expect, it } from "vitest";

import { ruleTermsDescription, termsAddedByRules } from "#/lib/rule-terms";

describe("termsAddedByRules", () => {
	it("is every stored term when the user sent none", () => {
		expect(termsAddedByRules(undefined, ["3 ply"])).toEqual(["3 ply"]);
	});

	it("leaves out the terms the user typed, in any casing", () => {
		expect(
			termsAddedByRules(["Recycled", "bulk"], ["Recycled", "bulk", "3 ply"]),
		).toEqual(["3 ply"]);
		expect(termsAddedByRules(["RECYCLED"], ["recycled"])).toEqual([]);
	});
});

describe("ruleTermsDescription", () => {
	it("says nothing when no rule applied", () => {
		expect(ruleTermsDescription([])).toBeUndefined();
	});

	it("names one filter", () => {
		expect(ruleTermsDescription(["3 ply"])).toBe(
			"Item rules added the filter: 3 ply",
		);
	});

	it("names several filters", () => {
		expect(ruleTermsDescription(["3 ply", "recycled"])).toBe(
			"Item rules added the filters: 3 ply, recycled",
		);
	});
});
