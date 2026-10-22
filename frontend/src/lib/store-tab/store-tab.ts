/**
 * The contract every store's trolley handoff satisfies in the web app.
 *
 * The server cannot log in to a store, so each store's trolley is filled by a
 * bookmarklet running in the household's own logged-in tab on that store's
 * website. The web app only ever sees this interface — never a store's
 * trolley calls — so the "Send to …" sheet, the bookmark and the status are
 * the same code for every store. The backend's side is store-agnostic
 * already (`/api/trolley-handoffs`, `/api/store-tab/*`).
 *
 * Adding a store: write its self-contained fill script, describe it as a
 * `StoreTab`, and list it in `store-tabs.ts`.
 */
import type { StoreId } from "#/lib/api/products";
import type { DeliveryReport } from "#/lib/store-tab/reserve-woolworths-delivery-window";

/** What every store's bookmarklet is built with. */
export interface FillTrolleyConfig {
	/** Our backend, e.g. `https://grocery.example.com` (no trailing slash). */
	apiBaseUrl: string;
	/** `STORE_TAB_SECRET`, from `GET /api/trolley-handoffs/store-tab-secret`. */
	secret: string;
}

/** What one bookmarklet run did, also shown to the person. */
export interface FillTrolleyResult {
	/** `null` when nothing was waiting or the page was not ready. */
	handoffId: string | null;
	added: string[];
	failed: { productId: string; problem: string }[];
	delivery: DeliveryReport | null;
	message: string;
	/**
	 * Offered when something was added: the store's checkout page (a path on
	 * its website) and the question asked before going there. The person
	 * reviews the order and pays on the store's website; nothing here places
	 * an order or touches payment.
	 */
	checkout: { path: string; question: string } | null;
}

/** One store's trolley handoff, as the web app uses it. */
export interface StoreTab {
	store: StoreId;
	/** The name a person reads, e.g. "Coles". */
	storeName: string;
	/** The page "Send and open …" opens on the store's website. */
	trolleyUrl: string;
	/** The bookmark's label, e.g. "Fill Coles trolley". */
	bookmarkName: string;
	/** Whether the bookmark reserves a delivery time itself. */
	reservesDelivery: boolean;
	/** The bookmarklet's `javascript:` address. */
	buildBookmarklet: (config: FillTrolleyConfig) => string;
}
