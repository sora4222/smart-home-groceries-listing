/**
 * How the Select item list looks, kept in this browser's local storage
 * (Settings › Display). Nothing here reaches the server.
 */

/** `full` is the picture at the store's own size. */
export type ImageSize = "sm" | "md" | "lg" | "full";

export interface DisplaySettings {
	imageSize: ImageSize;
	/** Show the struck-through price a reduced product used to cost. */
	showWasPrice: boolean;
	/** Show products the store has run out of. */
	showUnavailable: boolean;
}

export const DEFAULT_DISPLAY: DisplaySettings = {
	imageSize: "md",
	showWasPrice: true,
	showUnavailable: true,
};

const KEY = "grocery.display-settings";
const CHANGED = "grocery:display-settings";

const SIZES: ImageSize[] = ["sm", "md", "lg", "full"];

/** Stored settings, with defaults for anything missing or unreadable. */
export function parseDisplay(raw: string | null): DisplaySettings {
	let stored: Partial<Record<keyof DisplaySettings, unknown>> = {};
	try {
		stored = raw ? JSON.parse(raw) : {};
	} catch {
		// Unreadable: start again from the defaults.
	}
	return {
		imageSize: SIZES.includes(stored.imageSize as ImageSize)
			? (stored.imageSize as ImageSize)
			: DEFAULT_DISPLAY.imageSize,
		showWasPrice:
			typeof stored.showWasPrice === "boolean"
				? stored.showWasPrice
				: DEFAULT_DISPLAY.showWasPrice,
		showUnavailable:
			typeof stored.showUnavailable === "boolean"
				? stored.showUnavailable
				: DEFAULT_DISPLAY.showUnavailable,
	};
}

/** The last save storage refused, kept so it lasts until reload. */
let unsaved: string | null = null;

/** The raw stored text; `null` when there is none or storage is off. */
export function readDisplayRaw(): string | null {
	if (unsaved !== null) return unsaved;
	try {
		return window.localStorage.getItem(KEY);
	} catch {
		return null;
	}
}

/** Saves the settings and tells every open page they changed. */
export function saveDisplay(settings: DisplaySettings): void {
	const raw = JSON.stringify(settings);
	try {
		window.localStorage.setItem(KEY, raw);
		unsaved = null;
	} catch {
		// Storage is off or full; the change lasts until reload.
		unsaved = raw;
	}
	window.dispatchEvent(new Event(CHANGED));
}

/** Calls `onChange` when the settings change here or in another tab. */
export function subscribeDisplay(onChange: () => void): () => void {
	window.addEventListener(CHANGED, onChange);
	window.addEventListener("storage", onChange);
	return () => {
		window.removeEventListener(CHANGED, onChange);
		window.removeEventListener("storage", onChange);
	};
}
