/**
 * The Analysis page's address: which view and which filters, kept in the
 * URL so a view can be bookmarked and Back works.
 *
 * Anything unknown in the URL is dropped rather than refused, so an old or
 * hand-typed link still opens the page.
 */
import type { SpendingFilters, SpendingPeriod, StoreId } from "#/lib/api";
import { type RangePreset, rangeDays } from "#/lib/date-range";

/** The four views of the spending analysis. */
export type AnalysisView = "time" | "item" | "store" | "category";

export interface AnalysisSearch {
	view?: AnalysisView;
	range?: RangePreset;
	/** Custom range only: "YYYY-MM-DD". */
	from?: string;
	to?: string;
	store?: StoreId;
	item?: string;
	category?: string;
	period?: SpendingPeriod;
}

const VIEWS: AnalysisView[] = ["time", "item", "store", "category"];
const RANGES: RangePreset[] = ["week", "month", "quarter", "all", "custom"];
const STORES: StoreId[] = ["woolworths", "coles"];
const PERIODS: SpendingPeriod[] = ["week", "month", "quarter"];
const DAY = /^\d{4}-\d{2}-\d{2}$/;

/** Keeps only the parts of `raw` the page understands. */
export function validateAnalysisSearch(
	raw: Record<string, unknown>,
): AnalysisSearch {
	const search: AnalysisSearch = {};
	const oneOf = <T extends string>(value: unknown, allowed: T[]) =>
		allowed.includes(value as T) ? (value as T) : undefined;
	const text = (value: unknown) =>
		typeof value === "string" && value.trim() !== ""
			? value.slice(0, 200)
			: undefined;
	const day = (value: unknown) =>
		typeof value === "string" && DAY.test(value) ? value : undefined;

	search.view = oneOf(raw.view, VIEWS);
	search.range = oneOf(raw.range, RANGES);
	search.from = day(raw.from);
	search.to = day(raw.to);
	search.store = oneOf(raw.store, STORES);
	search.item = text(raw.item);
	search.category = text(raw.category);
	search.period = oneOf(raw.period, PERIODS);
	return Object.fromEntries(
		Object.entries(search).filter(([, value]) => value !== undefined),
	) as AnalysisSearch;
}

/** What to ask the backend for, on `today`. */
export function filtersFor(
	search: AnalysisSearch,
	today: Date,
): SpendingFilters {
	const days = rangeDays(search.range ?? "all", today, {
		from: search.from,
		to: search.to,
	});
	return {
		...days,
		store: search.store,
		item: search.item,
		category: search.category,
		period: search.period ?? "month",
	};
}
