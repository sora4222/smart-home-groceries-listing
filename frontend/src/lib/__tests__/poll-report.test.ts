import { describe, expect, it } from "vitest";

import { pollReportMessage } from "#/lib/poll-report";

const nothing = { recorded: 0, already_seen: 0, blank: 0, not_deleted: 0 };

describe("pollReportMessage", () => {
	it("says nothing new when nothing was found", () => {
		expect(pollReportMessage(nothing)).toBe("Nothing new.");
		expect(pollReportMessage({ ...nothing, already_seen: 2 })).toBe(
			"Nothing new.",
		);
	});

	it("counts new items, with the right plural", () => {
		expect(pollReportMessage({ ...nothing, recorded: 1 })).toBe(
			"1 new item found.",
		);
		expect(pollReportMessage({ ...nothing, recorded: 3 })).toBe(
			"3 new items found.",
		);
	});

	it("mentions empty tasks and tasks Google would not remove", () => {
		expect(
			pollReportMessage({ ...nothing, recorded: 1, blank: 1, not_deleted: 2 }),
		).toBe(
			"1 new item found. 1 empty task left alone. 2 tasks could not be removed from Google — will try again.",
		);
	});
});
