/**
 * Picks the Woolworths delivery window to reserve, from the windows the
 * website lists (`GET /api/v3/ui/fulfilment/windows`, live 2026-10-02).
 *
 * Rule: start from the day asked for — or the day after the store's own
 * "today" when none was — and use the first day that has a matching window.
 * On that day take the cheapest, then the earliest start, then the shortest
 * window. Express ("delivery now") windows are never picked.
 *
 * Must stay **self-contained** (no imports, no outside variables): the
 * bookmarklet carries its source text. See `fill-woolworths-trolley.ts`.
 */

/** One delivery window, as the website lists it. */
export interface WoolworthsWindow {
	Id: number;
	Available: boolean;
	/** The store's own words, e.g. "7am - 10am". */
	TimeWindow: string;
	/** Store-local, e.g. "2026-10-03T07:00:00.0000000". */
	StartDateTime: string;
	EndDateTime: string;
	SalePrice: number;
	TimeWindowDurationHours?: number;
	IsExpress?: boolean;
}

/** One day of windows, as the website lists it. */
export interface WoolworthsDay {
	/** e.g. "2026-10-03T00:00:00.0000000". */
	Date: string;
	Available: boolean;
	Times: WoolworthsWindow[];
}

export type TimeOfDay = "any" | "morning" | "afternoon" | "evening";

/** What the household asked for; `date` null = the store's next day. */
export interface DeliveryWanted {
	date: string | null;
	time_of_day: TimeOfDay;
}

/** The window to reserve, and whether it is on a later day than asked. */
export interface ChosenWindow {
	window: WoolworthsWindow;
	/** "YYYY-MM-DD". */
	date: string;
	wantedDate: string;
	movedToLaterDay: boolean;
}

/** Picks the window to reserve, or `null` when no day has a matching one. */
export function chooseWoolworthsWindow(
	days: WoolworthsDay[],
	wanted: DeliveryWanted,
	storeToday: string,
): ChosenWindow | null {
	const dayOf = (dateTime: string) => dateTime.slice(0, 10);
	const hourOf = (dateTime: string) => Number(dateTime.slice(11, 13));
	const nextDay = (day: string) => {
		const date = new Date(`${day.slice(0, 10)}T00:00:00Z`);
		date.setUTCDate(date.getUTCDate() + 1);
		return date.toISOString().slice(0, 10);
	};
	const fitsTimeOfDay = (start: string) => {
		const hour = hourOf(start);
		switch (wanted.time_of_day) {
			case "morning":
				return hour < 12;
			case "afternoon":
				return hour >= 12 && hour < 17;
			case "evening":
				return hour >= 17;
			default:
				return true;
		}
	};
	const wantedDate = wanted.date ?? nextDay(storeToday);
	const sorted = [...days].sort((a, b) => a.Date.localeCompare(b.Date));
	for (const day of sorted) {
		const date = dayOf(day.Date);
		if (date < wantedDate || !day.Available) continue;
		const candidates = day.Times.filter(
			(w) => w.Available && !w.IsExpress && fitsTimeOfDay(w.StartDateTime),
		).sort(
			(a, b) =>
				a.SalePrice - b.SalePrice ||
				a.StartDateTime.localeCompare(b.StartDateTime) ||
				(a.TimeWindowDurationHours ?? 0) - (b.TimeWindowDurationHours ?? 0),
		);
		if (candidates.length > 0) {
			return {
				window: candidates[0],
				date,
				wantedDate,
				movedToLaterDay: date !== wantedDate,
			};
		}
	}
	return null;
}
