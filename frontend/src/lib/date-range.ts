/**
 * The Analysis page's date ranges, as the household's own calendar days.
 *
 * "This week" starts on Monday, "This quarter" on 1 January, April, July or
 * October. Each runs to today. "All time" has no limits; "Custom" uses the
 * days the household typed in.
 */

/** A date-range choice on the Analysis page. */
export type RangePreset = "week" | "month" | "quarter" | "all" | "custom";

export const RANGE_PRESETS: ReadonlyArray<{
	value: RangePreset;
	label: string;
}> = [
	{ value: "week", label: "This week" },
	{ value: "month", label: "This month" },
	{ value: "quarter", label: "This quarter" },
	{ value: "all", label: "All time" },
	{ value: "custom", label: "Choose days" },
];

/** First and last day, "YYYY-MM-DD"; either may be missing. */
export interface DayRange {
	from?: string;
	to?: string;
}

/** `date` as the local "YYYY-MM-DD". */
export function isoDay(date: Date): string {
	const month = String(date.getMonth() + 1).padStart(2, "0");
	const day = String(date.getDate()).padStart(2, "0");
	return `${date.getFullYear()}-${month}-${day}`;
}

/**
 * The days `preset` covers on `today`. `custom` passes `chosen` through;
 * `all` has no limits.
 */
export function rangeDays(
	preset: RangePreset,
	today: Date,
	chosen: DayRange = {},
): DayRange {
	const to = isoDay(today);
	switch (preset) {
		case "week": {
			const monday = new Date(today);
			monday.setDate(today.getDate() - ((today.getDay() + 6) % 7));
			return { from: isoDay(monday), to };
		}
		case "month":
			return {
				from: isoDay(new Date(today.getFullYear(), today.getMonth(), 1)),
				to,
			};
		case "quarter": {
			const firstMonth = Math.floor(today.getMonth() / 3) * 3;
			return {
				from: isoDay(new Date(today.getFullYear(), firstMonth, 1)),
				to,
			};
		}
		case "all":
			return {};
		case "custom":
			return { from: chosen.from, to: chosen.to };
	}
}
