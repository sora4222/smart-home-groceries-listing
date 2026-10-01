/**
 * Trolley handoffs: `/api/trolley-handoffs`.
 *
 * The server cannot log in to Woolworths, so filling the trolley is handed to
 * the household's own logged-in Woolworths tab: the web app creates a
 * handoff here, the "Fill trolley" bookmarklet claims it on
 * woolworths.com.au, and the web app reads back what happened.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";

/** Where a handoff is in its life. */
export type TrolleyHandoffStatus =
	| "waiting_for_store_tab"
	| "claimed_by_store_tab"
	| "filled"
	| "filled_with_problems"
	| "replaced";

/** One list item's product inside a handoff. */
export interface TrolleyHandoffLine {
	grocery_item_id: string;
	product_id: string;
	name: string;
	quantity: number;
	/** `null` until the store tab reports back. */
	outcome: "added" | "failed" | null;
	problem: string | null;
}

/** Chosen products waiting for (or already put in) a store's trolley. */
export interface TrolleyHandoff {
	id: string;
	store: StoreId;
	store_name: string;
	status: TrolleyHandoffStatus;
	created_at: string;
	expires_at: string;
	claimed_at: string | null;
	reported_at: string | null;
	lines: TrolleyHandoffLine[];
}

export const trolleyHandoffsApi = {
	/** Hands every product chosen at `store` to the store tab. */
	create: (store: StoreId) =>
		request<TrolleyHandoff>("/api/trolley-handoffs", {
			method: "POST",
			body: JSON.stringify({ store }),
		}),
	/** A handoff and each line's outcome. */
	get: (id: string) => request<TrolleyHandoff>(`/api/trolley-handoffs/${id}`),
	/** The secret the "Fill trolley" bookmarklet is built with. */
	storeTabSecret: () =>
		request<{ secret: string }>("/api/trolley-handoffs/store-tab-secret"),
};

/** The handoff is finished: the store tab reported, or it was replaced. */
export function isHandoffFinished(handoff: TrolleyHandoff): boolean {
	return (
		handoff.status === "filled" ||
		handoff.status === "filled_with_problems" ||
		handoff.status === "replaced"
	);
}
