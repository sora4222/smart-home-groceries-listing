import { describe, expect, it } from "vitest";

import {
	describeSource,
	describeUser,
	formatLoggedAt,
	statusTone,
} from "#/lib/access-log-display";
import { accessLogQuery } from "#/lib/api/access-logs";

describe("access log display", () => {
	it("names a request with no session as not signed in", () => {
		expect(describeUser("unauthenticated")).toBe("Not signed in");
		expect(describeUser("user_abc")).toBe("user_abc");
	});

	it("shows the connection address, and a forwarded claim beside it", () => {
		expect(describeSource({ source_ip: "10.0.0.2", forwarded_for: null })).toBe(
			"10.0.0.2",
		);
		expect(
			describeSource({ source_ip: "10.0.0.2", forwarded_for: "203.0.113.9" }),
		).toBe("10.0.0.2 (says 203.0.113.9)");
		expect(describeSource({ source_ip: null, forwarded_for: null })).toBe(
			"unknown",
		);
	});

	it("colours server failures red and refusals outlined", () => {
		expect(statusTone(200)).toBe("secondary");
		expect(statusTone(101)).toBe("secondary");
		expect(statusTone(404)).toBe("outline");
		expect(statusTone(499)).toBe("outline");
		expect(statusTone(500)).toBe("destructive");
	});

	it("formats the time to the second", () => {
		const text = formatLoggedAt("2026-10-02T04:40:16Z", "en-AU");
		expect(text).toMatch(/\d{2}:\d{2}:\d{2}/);
		expect(text).toMatch(/Oct/);
	});
});

describe("accessLogQuery", () => {
	it("always says whether to hide health checks", () => {
		expect(accessLogQuery({ hideHealthChecks: true })).toBe(
			"hide_health_checks=true",
		);
	});

	it("adds the cursor for an older page", () => {
		expect(accessLogQuery({ before: 42, hideHealthChecks: false })).toBe(
			"hide_health_checks=false&before=42",
		);
	});
});
