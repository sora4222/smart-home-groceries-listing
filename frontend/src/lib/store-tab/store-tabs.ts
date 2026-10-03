/**
 * Every store the household can send a trolley to, as a {@link StoreTab}.
 *
 * The one place that names a store's fill script, so the rest of the web app
 * picks a store by its id and nothing else.
 */
import type { StoreId } from "#/lib/api/products";
import { buildBookmarklet } from "#/lib/store-tab/bookmarklet";
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";
import { fillColesTrolley } from "#/lib/store-tab/fill-coles-trolley";
import { fillWoolworthsTrolley } from "#/lib/store-tab/fill-woolworths-trolley";
import { reserveWoolworthsDeliveryWindow } from "#/lib/store-tab/reserve-woolworths-delivery-window";
import type { StoreTab } from "#/lib/store-tab/store-tab";

/** Woolworths: fills the trolley and reserves the delivery time. */
export const WOOLWORTHS_TAB: StoreTab = {
	store: "woolworths",
	storeName: "Woolworths",
	trolleyUrl: "https://www.woolworths.com.au/shop/mytrolley",
	bookmarkName: "Fill Woolworths trolley",
	reservesDelivery: true,
	buildBookmarklet: (config) =>
		buildBookmarklet(fillWoolworthsTrolley, config, {
			chooseWindow: chooseWoolworthsWindow,
			reserveDeliveryWindow: reserveWoolworthsDeliveryWindow,
		}),
};

/** Coles: fills the trolley; the delivery time is picked on Coles. */
export const COLES_TAB: StoreTab = {
	store: "coles",
	storeName: "Coles",
	trolleyUrl: "https://www.coles.com.au/",
	bookmarkName: "Fill Coles trolley",
	reservesDelivery: false,
	buildBookmarklet: (config) => buildBookmarklet(fillColesTrolley, config),
};

/** Each store's trolley handoff, by store id. */
export const STORE_TABS: Record<StoreId, StoreTab> = {
	woolworths: WOOLWORTHS_TAB,
	coles: COLES_TAB,
};
