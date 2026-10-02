/**
 * The desktop app's commands, typed. Each one is granted to the server's
 * page in `desktop/src-tauri/src/server_access.rs`.
 */
import type { StoreId } from "#/lib/api/products";
import { invokeDesktop } from "#/lib/desktop/bridge";

/** What the desktop app can do at one store today. */
export interface StoreAbilities {
	store: StoreId;
	/** Open the store window to log in. */
	login: boolean;
	/** Fill the trolley in the store window. */
	fill_trolley: boolean;
	/** Show the store's checkout page. */
	checkout: boolean;
}

/** The desktop app's version and abilities. */
export interface DesktopAbilities {
	version: string;
	stores: StoreAbilities[];
}

export const desktop = {
	/** What the app can do at each store. */
	abilities: () => invokeDesktop<DesktopAbilities>("desktop_abilities"),
	/** Shows the store's website in its window, to log in. */
	openStore: (store: StoreId) => invokeDesktop<void>("open_store", { store }),
	/** Runs the fill program on the store's trolley page in its window. */
	fillTrolley: (store: StoreId, program: string) =>
		invokeDesktop<void>("fill_store_trolley", { store, program }),
	/** Shows the store's checkout page in its window. */
	openCheckout: (store: StoreId) =>
		invokeDesktop<void>("open_store_checkout", { store }),
	/** A desktop notification. */
	notify: (title: string, body: string) =>
		invokeDesktop<void>("notify", { title, body }),
};

/** The abilities for `store`, or none. */
export function abilitiesAt(
	abilities: DesktopAbilities | null,
	store: StoreId,
): StoreAbilities | null {
	return abilities?.stores.find((entry) => entry.store === store) ?? null;
}
