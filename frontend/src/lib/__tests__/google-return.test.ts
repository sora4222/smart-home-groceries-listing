import { describe, expect, it } from "vitest";

import { googleReturn } from "#/lib/google-return";

describe("googleReturn", () => {
	it("keeps Google's code, state and error", () => {
		expect(googleReturn({ code: "c", state: "s", error: "e" })).toEqual({
			code: "c",
			state: "s",
			error: "e",
		});
	});

	it("drops anything else, blanks and non-strings", () => {
		expect(googleReturn({ code: "", state: 3, other: "x" })).toEqual({});
	});
});
