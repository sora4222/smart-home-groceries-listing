import { describe, expect, it } from "vitest";
import { liveDays, windowAt } from "#/lib/__tests__/woolworths-fixtures";
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";

const today = "2026-10-02T00:00:00.0000000";

describe("chooseWoolworthsWindow", () => {
	it("takes the next day's earliest window by default when prices are equal", () => {
		const chosen = chooseWoolworthsWindow(
			liveDays(),
			{ date: null, time_of_day: "any" },
			today,
		);

		expect(chosen?.date).toBe("2026-10-03");
		expect(chosen?.window.Id).toBe(948311);
		expect(chosen?.movedToLaterDay).toBe(false);
	});

	it("keeps to the part of the day asked for", () => {
		const pick = (time_of_day: "morning" | "afternoon" | "evening") =>
			chooseWoolworthsWindow(liveDays(), { date: null, time_of_day }, today)
				?.window.Id;

		expect(pick("morning")).toBe(948311);
		expect(pick("afternoon")).toBe(948304);
		expect(pick("evening")).toBe(948300);
	});

	it("takes the cheapest first, then the earliest", () => {
		const days = liveDays();
		days[1].Times.push(windowAt(1, "2026-10-03", 18, 3, 9));

		expect(
			chooseWoolworthsWindow(days, { date: null, time_of_day: "any" }, today)
				?.window.Id,
		).toBe(1);
	});

	it("never picks unavailable or express windows", () => {
		const days = liveDays();
		days[1].Times = [
			windowAt(1, "2026-10-03", 4, 1, 5, { Available: false }),
			windowAt(2, "2026-10-03", 5, 1, 5, { IsExpress: true }),
			windowAt(3, "2026-10-03", 9, 3),
		];

		expect(
			chooseWoolworthsWindow(days, { date: null, time_of_day: "any" }, today)
				?.window.Id,
		).toBe(3);
	});

	it("moves to the next day with a matching window, and says so", () => {
		const days = liveDays();
		days[1].Times = days[1].Times.filter(
			(w) => !w.StartDateTime.includes("T17"),
		);
		days.push({
			Date: "2026-10-04T00:00:00.0000000",
			Available: true,
			Times: [windowAt(9, "2026-10-04", 18, 3)],
		});

		const chosen = chooseWoolworthsWindow(
			days,
			{ date: "2026-10-03", time_of_day: "evening" },
			today,
		);

		expect(chosen?.date).toBe("2026-10-04");
		expect(chosen?.movedToLaterDay).toBe(true);
		expect(chosen?.wantedDate).toBe("2026-10-03");
	});

	it("gives up when no day has a matching window", () => {
		expect(
			chooseWoolworthsWindow([], { date: null, time_of_day: "any" }, today),
		).toBeNull();
	});
});
