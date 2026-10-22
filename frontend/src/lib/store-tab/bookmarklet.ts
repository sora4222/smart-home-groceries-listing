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
 * Builds the program shared by bookmarks and the desktop store window.
 * `log` reports completion without opening checkout or showing a dialog.
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
	finish: "alert" | "log" = "alert",
): string {
	const settings = JSON.stringify({
		apiBaseUrl: config.apiBaseUrl.replace(/\/+$/, ""),
		secret: config.secret,
	});
	const helperSource = `{${Object.entries(helpers)
		.map(([name, helper]) => `${name}:${helper.toString()}`)
		.join(",")}}`;
	const report =
		finish === "log"
			? ".then(function(r){console.info(r.message)},function(){console.error('Fill trolley failed')});void 0"
			: ".then(function(r){if(!r.checkout){alert(r.message);return}" +
				"if(confirm(r.message+'\\n\\n'+r.checkout.question))location.assign(r.checkout.path)}," +
				"function(e){alert('Fill trolley failed: '+e.message)});void 0";
	return `(${fill.toString()})(${settings},${helperSource})${report}`;
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
