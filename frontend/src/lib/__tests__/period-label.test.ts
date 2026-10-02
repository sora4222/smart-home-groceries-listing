import { describe, expect, it } from "vitest";

import { periodLabel, periodTick } from "#/lib/period-label";

describe("periodLabel", () => {
	it("names weeks, months and quarters", () => {
		expect(periodLabel("week", "2026-09-28")).toBe("Week of 28 Sep 2026");
		expect(periodLabel("month", "2026-09-01")).toBe("Sep 2026");
		expect(periodLabel("quarter", "2026-07-01")).toBe("Jul–Sep 2026");
		expect(periodLabel("quarter", "2026-10-01")).toBe("Oct–Dec 2026");
	});

	it("shows something unreadable as it came", () => {
		expect(periodLabel("month", "soon")).toBe("soon");
	});
});

describe("periodTick", () => {
	it("is short", () => {
		expect(periodTick("month", "2026-09-01")).toBe("Sep 26");
		expect(periodTick("week", "2026-09-28")).toBe("28 Sep");
	});
});
