/**
 * Settings › Delivery: `GET` and `PUT /api/delivery-settings`.
 *
 * Each store's delivery fee rules, typed once by the household (the app
 * cannot read a store's fees without the household's own login), and the
 * order planner's default mode and delivery spending cap. Money is a decimal
 * string; `null` means not set.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";

/** How the order screen ranks the ways to split an order between stores. */
export type OrderMode =
	| "minimise_total"
	| "minimise_delivery"
	| "woolworths_only"
	| "coles_only"
	| "manual";

/** One store's delivery fee rules. */
export interface StoreFeeRules {
	store: StoreId;
	/** Only in answers. */
	store_name?: string;
	/** What one delivery costs; `null` = not set yet. */
	delivery_fee: string | null;
	/** An order this big or bigger delivers free; `null` = never free. */
	free_delivery_over: string | null;
	/** The store refuses a smaller order; `null` = no minimum. */
	minimum_order: string | null;
}

/** Everything on Settings › Delivery. */
export interface DeliverySettings {
	stores: StoreFeeRules[];
	mode: OrderMode;
	/** Options with more delivery fees than this are not recommended. */
	max_delivery_spend: string | null;
}

export const deliverySettingsApi = {
	/** The saved settings, every store present. */
	get: () => request<DeliverySettings>("/api/delivery-settings"),
	/** Saves them and answers with what is now saved. */
	save: (settings: DeliverySettings) =>
		request<DeliverySettings>("/api/delivery-settings", {
			method: "PUT",
			body: JSON.stringify(settings),
		}),
};
