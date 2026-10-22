/**
 * Turns a store's fill script into a `javascript:` bookmarklet: a link the
 * household drags to the bookmarks bar once, then presses on the store's
 * website after "Send to …" in the app.
 *
 * The link carries the script's source text plus the backend URL and the
 * store-tab secret, and shows the result. When the script offers checkout,
 * the result is shown as a question instead ("Go to checkout now?"): OK opens
 * the store's checkout page, where the person reviews and pays. The app
 * stops there and never places the order. Rebuild it (drag it
 * again) if the backend URL or `STORE_TAB_SECRET` changes.
 */
import type { FillTrolleyConfig, StoreTab } from "#/lib/store-tab/store-tab";

/**
 * Where the desktop program keeps its progress on the store page:
 * `{state: "running"}`, then `{state: "completed" | "failed", message}`.
 * The store page cannot call the desktop app, so the app reads this value
 * instead (`desktop/src-tauri/src/fill_status.rs`).
 */
export const FILL_PROGRESS_GLOBAL = "__groceryFill";

const PROGRESS = `window.${FILL_PROGRESS_GLOBAL}`;

/** Keeps the result in {@link FILL_PROGRESS_GLOBAL}; no checkout, no dialog. */
const REPORT_PROGRESS =
	`.then(function(r){${PROGRESS}={state:'completed',message:String(r.message)}},` +
	`function(e){${PROGRESS}={state:'failed',message:'Fill trolley failed: '+(e&&e.message||e)}});void 0`;

/** Shows the result, or asks the checkout question when one is offered. */
const SHOW_RESULT =
	".then(function(r){if(!r.checkout){alert(r.message);return}" +
	"if(confirm(r.message+'\\n\\n'+r.checkout.question))location.assign(r.checkout.path)}," +
	"function(e){alert('Fill trolley failed: '+e.message)});void 0";

/** What a fill program is built from. */
type FillSource = Pick<StoreTab, "fill" | "helpers">;

/**
 * Calls the store's fill as `fill(settings, helpers)`. Each helper is
 * self-contained too. They are handed in as an argument, so a minifier
 * renaming them cannot break the program.
 */
function fillCall({ fill, helpers }: FillSource, config: FillTrolleyConfig) {
	const settings = JSON.stringify({
		apiBaseUrl: config.apiBaseUrl.replace(/\/+$/, ""),
		secret: config.secret,
	});
	const helperSource = `{${Object.entries(helpers)
		.map(([name, helper]) => `${name}:${helper.toString()}`)
		.join(",")}}`;
	return `(${fill.toString()})(${settings},${helperSource})`;
}

/** The program for the desktop store window: progress only, no dialogs. */
export function desktopProgram(
	tab: FillSource,
	config: FillTrolleyConfig,
): string {
	return `${PROGRESS}={state:'running'};${fillCall(tab, config)}${REPORT_PROGRESS}`;
}

/** The browser bookmark's `javascript:` address, with its checkout question. */
export function bookmarklet(
	tab: FillSource,
	config: FillTrolleyConfig,
): string {
	return `javascript:${encodeURIComponent(fillCall(tab, config) + SHOW_RESULT)}`;
}

/** The absolute backend URL a script on another website must call. */
export function absoluteApiBaseUrl(
	apiBaseUrl: string,
	pageOrigin: string,
): string {
	return new URL(apiBaseUrl, pageOrigin).toString().replace(/\/+$/, "");
}
