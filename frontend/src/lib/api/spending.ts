/**
 * The spending analysis: `/api/spending`.
 *
 * One request returns every view (over time, by item, by store, by
 * category) for the filters, so switching views asks nothing new. Days are
 * the household's own: the browser's time zone is sent with every request.
 * Money is a decimal string.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";
import type { PurchaseOrder } from "#/lib/api/purchases";

/** How "Over time" groups spend. */
export type SpendingPeriod = "week" | "month" | "quarter";

/** The page's filters. Every one is optional. */
export interface SpendingFilters {
	/** First day, "YYYY-MM-DD". */
	from?: string;
	/** Last day, included. */
	to?: string;
	store?: StoreId;
	/** Part of an item's name. */
	item?: string;
	category?: string;
	period?: SpendingPeriod;
}

export interface PeriodSpend {
	/** The bucket's first day, "YYYY-MM-DD". */
	start: string;
	total: string;
}

export interface ItemSpend {
	item_name: string;
	quantity: number;
	times_bought: number;
	total: string;
}

export interface StoreSpend {
	store: StoreId;
	store_name: string;
	items_total: string;
	delivery_total: string;
	total: string;
}

export interface CategorySpend {
	category: string;
	total: string;
}

/** Every view of the Analysis page. */
export interface SpendingReport {
	/** The newest shop, whatever the filters say. */
	latest_order: PurchaseOrder | null;
	/** Every category any purchase is in, A to Z. */
	categories: string[];
	items_total: string;
	delivery_total: string;
	total: string;
	over_time: PeriodSpend[];
	by_item: ItemSpend[];
	by_store: StoreSpend[];
	by_category: CategorySpend[];
}

/** What an item cost on one shop. */
export interface ItemPrice {
	bought_at: string;
	store: StoreId;
	store_name: string;
	product_name: string;
	brand: string | null;
	package_size: string | null;
	quantity: number;
	unit_price: string;
	total_price: string;
}

/** The browser's time zone, e.g. "Australia/Sydney". */
export function browserTimeZone(): string {
	return Intl.DateTimeFormat().resolvedOptions().timeZone ?? "UTC";
}

/** `filters` as a query string, blanks left out, the time zone added. */
export function spendingQuery(
	filters: SpendingFilters,
	tz = browserTimeZone(),
): string {
	const params = new URLSearchParams();
	for (const [key, value] of Object.entries(filters)) {
		if (typeof value === "string" && value.trim() !== "") {
			params.set(key, value);
		}
	}
	params.set("tz", tz);
	return params.toString();
}

export const spendingApi = {
	/** Every view for `filters`. */
	get: (filters: SpendingFilters) =>
		request<SpendingReport>(`/api/spending?${spendingQuery(filters)}`),
	/** One item's price on every shop, oldest first. */
	itemPrices: (name: string) =>
		request<ItemPrice[]>(
			`/api/spending/item-prices?${new URLSearchParams({ name })}`,
		),
};
