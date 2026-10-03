import { describe, expect, it } from "vitest";

import { isoDay, rangeDays } from "#/lib/date-range";

// Friday 2 October 2026, local time.
const today = new Date(2026, 9, 2, 15, 30);

describe("rangeDays", () => {
	it("starts this week on Monday", () => {
		expect(rangeDays("week", today)).toEqual({
			from: "2026-09-28",
			to: "2026-10-02",
		});
	});

	it("treats Sunday as the end of the week", () => {
		expect(rangeDays("week", new Date(2026, 9, 4))).toEqual({
			from: "2026-09-28",
			to: "2026-10-04",
		});
	});

	it("starts this month on the first", () => {
		expect(rangeDays("month", today).from).toBe("2026-10-01");
	});

	it("starts this quarter in January, April, July or October", () => {
		expect(rangeDays("quarter", today).from).toBe("2026-10-01");
		expect(rangeDays("quarter", new Date(2026, 4, 15)).from).toBe("2026-04-01");
	});

	it("has no limits for all time", () => {
		expect(rangeDays("all", today)).toEqual({});
	});

	it("passes chosen days through for a custom range", () => {
		expect(
			rangeDays("custom", today, { from: "2026-01-01", to: "2026-02-01" }),
		).toEqual({ from: "2026-01-01", to: "2026-02-01" });
	});
});

describe("isoDay", () => {
	it("pads the month and day", () => {
		expect(isoDay(new Date(2026, 0, 5))).toBe("2026-01-05");
	});
});
