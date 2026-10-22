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
import type { FillTrolleyConfig } from "#/lib/store-tab/store-tab";

/** A self-contained function: the bookmarklet carries its source text. */
// biome-ignore lint/suspicious/noExplicitAny: any script and helper shape is carried as text.
type SelfContained = (...args: any[]) => unknown;

/**
 * Where the desktop program keeps its progress on the store page:
 * `{state: "running"}`, then `{state: "completed" | "failed", message}`.
 * The store page cannot call the desktop app, so the app reads this value
 * instead (`desktop/src-tauri/src/fill_status.rs`).
 */
export const FILL_PROGRESS_GLOBAL = "__groceryFill";

/**
 * Builds the program shared by bookmarks and the desktop store window.
 * `report` keeps the progress in {@link FILL_PROGRESS_GLOBAL} without opening
 * checkout or showing a dialog.
 *
 * The fill function is called as `fill(settings, helpers)`.
 *
 * Each helper is self-contained too. They are handed in as an argument, so a
 * minifier renaming them cannot break the link.
 */
export function buildFillProgram(
	fill: SelfContained,
	config: FillTrolleyConfig,
	helpers: Record<string, SelfContained> = {},
	finish: "alert" | "report" = "alert",
): string {
	const settings = JSON.stringify({
		apiBaseUrl: config.apiBaseUrl.replace(/\/+$/, ""),
		secret: config.secret,
	});
	const helperSource = `{${Object.entries(helpers)
		.map(([name, helper]) => `${name}:${helper.toString()}`)
		.join(",")}}`;
	const progress = `window.${FILL_PROGRESS_GLOBAL}`;
	const start = finish === "report" ? `${progress}={state:'running'};` : "";
	const report =
		finish === "report"
			? `.then(function(r){${progress}={state:'completed',message:String(r.message)}},` +
				`function(e){${progress}={state:'failed',message:'Fill trolley failed: '+(e&&e.message||e)}});void 0`
			: ".then(function(r){if(!r.checkout){alert(r.message);return}" +
				"if(confirm(r.message+'\\n\\n'+r.checkout.question))location.assign(r.checkout.path)}," +
				"function(e){alert('Fill trolley failed: '+e.message)});void 0";
	return `${start}(${fill.toString()})(${settings},${helperSource})${report}`;
}

/** The browser bookmark retains its checkout confirmation dialog. */
export function buildBookmarklet(
	fill: SelfContained,
	config: FillTrolleyConfig,
	helpers: Record<string, SelfContained> = {},
): string {
	return `javascript:${encodeURIComponent(buildFillProgram(fill, config, helpers))}`;
}

/** The absolute backend URL a script on another website must call. */
export function absoluteApiBaseUrl(
	apiBaseUrl: string,
	pageOrigin: string,
): string {
	return new URL(apiBaseUrl, pageOrigin).toString().replace(/\/+$/, "");
}
