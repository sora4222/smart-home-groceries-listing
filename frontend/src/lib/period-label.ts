/**
 * Names for the "Over time" buckets: "Week of 28 Sep 2026", "Sep 2026",
 * "Jul–Sep 2026". A bucket is given by its first day, "YYYY-MM-DD".
 */
import type { SpendingPeriod } from "#/lib/api";

const MONTHS = [
	"Jan",
	"Feb",
	"Mar",
	"Apr",
	"May",
	"Jun",
	"Jul",
	"Aug",
	"Sep",
	"Oct",
	"Nov",
	"Dec",
];

/** The bucket starting on `start`, for people. */
export function periodLabel(period: SpendingPeriod, start: string): string {
	const [year, month, day] = start.split("-").map(Number);
	if (!year || !month || !day) return start;
	const name = MONTHS[month - 1];
	switch (period) {
		case "week":
			return `Week of ${day} ${name} ${year}`;
		case "month":
			return `${name} ${year}`;
		case "quarter":
			return `${name}–${MONTHS[(month + 1) % 12]} ${year}`;
	}
}

/** A shorter name for an axis: "28 Sep", "Sep 26", "Jul–Sep 26". */
export function periodTick(period: SpendingPeriod, start: string): string {
	const [year, month, day] = start.split("-").map(Number);
	if (!year || !month || !day) return start;
	const name = MONTHS[month - 1];
	const short = String(year).slice(2);
	switch (period) {
		case "week":
			return `${day} ${name}`;
		case "month":
			return `${name} ${short}`;
		case "quarter":
			return `${name}–${MONTHS[(month + 1) % 12]} ${short}`;
	}
}
