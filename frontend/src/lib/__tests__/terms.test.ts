import { describe, expect, it } from "vitest";

import { addTerm } from "#/lib/terms";

describe("addTerm", () => {
	it("appends a trimmed term", () => {
		expect(addTerm(["3 ply"], "  recycled ")).toEqual(["3 ply", "recycled"]);
	});

	it("ignores a blank draft", () => {
		expect(addTerm(["3 ply"], "   ")).toEqual(["3 ply"]);
	});

	it("ignores a repeat in any casing", () => {
		expect(addTerm(["3 ply"], "3 PLY")).toEqual(["3 ply"]);
	});

	it("stops at the limit", () => {
		const full = Array.from({ length: 10 }, (_, n) => `term ${n}`);
		expect(addTerm(full, "one more")).toEqual(full);
	});

	it("does not change the list it was given", () => {
		const terms = ["3 ply"];
		addTerm(terms, "recycled");
		expect(terms).toEqual(["3 ply"]);
	});
});
