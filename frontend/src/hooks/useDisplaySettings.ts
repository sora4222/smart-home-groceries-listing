import { useSyncExternalStore } from "react";

import {
	type DisplaySettings,
	parseDisplay,
	readDisplayRaw,
	subscribeDisplay,
} from "#/lib/display-settings";

/** The Select item list's display settings, following every change. */
export function useDisplaySettings(): DisplaySettings {
	// The raw string is the snapshot: it only changes when the settings do.
	const raw = useSyncExternalStore(
		subscribeDisplay,
		readDisplayRaw,
		() => null,
	);
	return parseDisplay(raw);
}
