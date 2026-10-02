import { describe, expect, it } from "vitest";

import { POLL_CHOICES, pollIntervalLabel } from "#/lib/poll-interval";

describe("pollIntervalLabel", () => {
	it("names each choice in words", () => {
		expect(pollIntervalLabel(60)).toBe("Every minute");
		expect(pollIntervalLabel(3600)).toBe("Every hour");
	});

	it("falls back to seconds for a gap not in the list", () => {
		expect(pollIntervalLabel(45)).toBe("Every 45 seconds");
	});

	it("only offers gaps the server accepts (30 seconds to an hour)", () => {
		for (const choice of POLL_CHOICES) {
			expect(choice.seconds).toBeGreaterThanOrEqual(30);
			expect(choice.seconds).toBeLessThanOrEqual(3600);
		}
	});
});
