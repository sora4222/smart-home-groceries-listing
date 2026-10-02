/**
 * Builds the fill program and the "Fill Woolworths trolley" bookmarklet: a
 * `javascript:` link the household drags to the bookmarks bar once, then
 * presses on woolworths.com.au after "Send to Woolworths" in the app. The
 * desktop app runs the same program in its store window instead.
 *
 * The link carries the source of {@link fillWoolworthsTrolley} and its
 * helpers (choosing and reserving the delivery window) plus the backend URL
 * and the store-tab secret, and shows the result in an alert.
 * Rebuild it (drag it again) if the backend URL or `STORE_TAB_SECRET` changes.
 */
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";
import {
	type FillTrolleyConfig,
	fillWoolworthsTrolley,
} from "#/lib/store-tab/fill-woolworths-trolley";
import { reserveWoolworthsDeliveryWindow } from "#/lib/store-tab/reserve-woolworths-delivery-window";

/** How the program tells the person the result: an alert in the browser
 * (bookmark), the console in the desktop app's store window, where the web
 * app follows the handoff and sends a notification instead. */
export type ProgramFinish = "alert" | "log";

/**
 * The fill program as plain JavaScript: {@link fillWoolworthsTrolley} with
 * its helpers, the backend URL and the store-tab secret. The bookmark wraps
 * it in a `javascript:` link; the desktop app runs it in its store window.
 */
export function buildFillWoolworthsTrolleyProgram(
	config: FillTrolleyConfig,
	finish: ProgramFinish,
): string {
	const settings = JSON.stringify({
		apiBaseUrl: config.apiBaseUrl.replace(/\/+$/, ""),
		secret: config.secret,
	});
	// Each function is self-contained; the helpers are handed in as an
	// argument, so a minifier renaming them cannot break the program.
	const helpers =
		`{chooseWindow:${chooseWoolworthsWindow.toString()},` +
		`reserveDeliveryWindow:${reserveWoolworthsDeliveryWindow.toString()}}`;
	const show =
		finish === "alert"
			? "function(r){alert(r.message)},function(e){alert('Fill trolley failed: '+e.message)}"
			: "function(r){console.info('[fill-trolley]',r.message)},function(e){console.error('[fill-trolley] failed',e)}";
	return `(${fillWoolworthsTrolley.toString()})(${settings},${helpers}).then(${show});void 0`;
}

/** The bookmarklet's `href`. */
export function buildFillWoolworthsTrolleyBookmarklet(
	config: FillTrolleyConfig,
): string {
	const program = buildFillWoolworthsTrolleyProgram(config, "alert");
	return `javascript:${encodeURIComponent(program)}`;
}

/** The absolute backend URL a script on another website must call. */
export function absoluteApiBaseUrl(
	apiBaseUrl: string,
	pageOrigin: string,
): string {
	return new URL(apiBaseUrl, pageOrigin).toString().replace(/\/+$/, "");
}
