import { describe, expect, it } from "vitest";

import { confidenceLabel } from "#/lib/triage-confidence";

describe("confidenceLabel", () => {
	it("rounds to a whole percent", () => {
		expect(confidenceLabel(0.856)).toBe("86% sure");
		expect(confidenceLabel(1)).toBe("100% sure");
		expect(confidenceLabel(0)).toBe("0% sure");
	});

	it("is null when the checker never answered", () => {
		expect(confidenceLabel(null)).toBeNull();
	});
});
