/**
 * The order planner: `GET /api/order-options`.
 *
 * The ways to buy the committed list — all at Woolworths, all at Coles, the
 * best mix, and the stores the order buys from now — each with delivery fees
 * from Settings › Delivery, best first under the mode. Using one sends its
 * `picks` to `api.selections.buyAt`. Money is a decimal string.
 */
import { request } from "#/lib/api/client";
import type { OrderMode } from "#/lib/api/delivery-settings";
import type { OrderLine, UnchosenItem } from "#/lib/api/order-review";
import type { StoreId } from "#/lib/api/products";
import type { OrderStorePick } from "#/lib/api/selections";

/** Which way of buying an option is. */
export type OrderOptionKind = "split" | "woolworths" | "coles" | "current";

/** What an option buys at one store, with its delivery. */
export interface OptionStore {
	store: StoreId;
	store_name: string;
	lines: OrderLine[];
	subtotal: string;
	delivery_fee: string;
	/** The fee is set in Settings › Delivery (otherwise counted as $0). */
	fee_known: boolean;
	free_delivery: boolean;
	/** The store will not deliver an order this small. */
	below_minimum: boolean;
	minimum_order: string | null;
}

/** An item an option cannot buy, and why. */
export interface MissingItem {
	grocery_item_id: string;
	name: string;
	reason: string;
}

/** One way to buy the order. */
export interface OrderOption {
	kind: OrderOptionKind;
	label: string;
	/** The best, and it can be placed as it is. */
	recommended: boolean;
	/** The order buys from these stores now. */
	is_current: boolean;
	stores: OptionStore[];
	missing: MissingItem[];
	items_total: string;
	delivery_total: string;
	total: string;
	complete: boolean;
	fees_known: boolean;
	meets_minimums: boolean;
	within_delivery_cap: boolean;
	picks: OrderStorePick[];
}

/** Every option, best first. */
export interface OrderOptions {
	mode: OrderMode;
	options: OrderOption[];
	unchosen: UnchosenItem[];
	/** The best mix was proven best (every combination tried). */
	exact: boolean;
	max_delivery_spend: string | null;
}

export const orderOptionsApi = {
	/** The options, ranked by `mode` or the saved one. */
	get: (mode?: OrderMode) =>
		request<OrderOptions>(
			`/api/order-options${mode ? `?mode=${encodeURIComponent(mode)}` : ""}`,
		),
};
