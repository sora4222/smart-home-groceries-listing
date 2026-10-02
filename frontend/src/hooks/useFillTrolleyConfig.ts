import { useEffect, useState } from "react";

import { api } from "#/lib/api";
import { API_BASE_URL } from "#/lib/config";
import { absoluteApiBaseUrl } from "#/lib/store-tab/bookmarklet";
import type { FillTrolleyConfig } from "#/lib/store-tab/fill-woolworths-trolley";

/** The fill program's settings, once the store-tab secret has arrived. */
export interface FillTrolleyConfigState {
	config: FillTrolleyConfig | null;
	/** True when the server has no `STORE_TAB_SECRET`. */
	failed: boolean;
}

/**
 * Fetches the store-tab secret and builds the settings the fill program
 * needs (the backend's absolute URL and the secret). Used by the bookmark
 * and by the desktop app's "Fill trolley in the app".
 */
export function useFillTrolleyConfig(): FillTrolleyConfigState {
	const [state, setState] = useState<FillTrolleyConfigState>({
		config: null,
		failed: false,
	});
	useEffect(() => {
		let cancelled = false;
		api.trolleyHandoffs
			.storeTabSecret()
			.then(({ secret }) => {
				if (cancelled) return;
				const apiBaseUrl = absoluteApiBaseUrl(
					API_BASE_URL,
					window.location.origin,
				);
				setState({ config: { apiBaseUrl, secret }, failed: false });
			})
			.catch((err) => {
				console.error("[trolley-handoff] no store-tab secret", err);
				if (!cancelled) setState({ config: null, failed: true });
			});
		return () => {
			cancelled = true;
		};
	}, []);
	return state;
}
