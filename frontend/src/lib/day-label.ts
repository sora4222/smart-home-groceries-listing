/**
 * Short, plain day labels for past purchases: "2 Oct 2026".
 *
 * Shown in the browser's own time zone, the same calendar the spending
 * analysis groups days by.
 */

const formatter = new Intl.DateTimeFormat("en-AU", {
	day: "numeric",
	month: "short",
	year: "numeric",
});

/** An ISO instant (or "YYYY-MM-DD" day) as "2 Oct 2026". */
export function dayLabel(value: string): string {
	const date = /^\d{4}-\d{2}-\d{2}$/.test(value)
		? new Date(`${value}T00:00:00`)
		: new Date(value);
	if (Number.isNaN(date.getTime())) return value;
	return formatter.format(date);
}
