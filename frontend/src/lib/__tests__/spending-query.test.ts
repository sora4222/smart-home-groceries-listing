import { describe, expect, it } from "vitest";

import { spendingQuery } from "#/lib/api/spending";

describe("spendingQuery", () => {
	it("leaves blank filters out and adds the time zone", () => {
		const query = new URLSearchParams(
			spendingQuery(
				{ from: "2026-09-01", item: "  ", store: "coles", period: "week" },
				"Australia/Sydney",
			),
		);
		expect(Object.fromEntries(query)).toEqual({
			from: "2026-09-01",
			store: "coles",
			period: "week",
			tz: "Australia/Sydney",
		});
	});
});
