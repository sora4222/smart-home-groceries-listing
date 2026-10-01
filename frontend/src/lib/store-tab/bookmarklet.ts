/**
 * Builds the "Fill Woolworths trolley" bookmarklet: a `javascript:` link the
 * household drags to the bookmarks bar once, then presses on
 * woolworths.com.au after "Send to Woolworths" in the app.
 *
 * The link carries the source of {@link fillWoolworthsTrolley} plus the
 * backend URL and the store-tab secret, and shows the result in an alert.
 * Rebuild it (drag it again) if the backend URL or `STORE_TAB_SECRET` changes.
 */
import {
	type FillTrolleyConfig,
	fillWoolworthsTrolley,
} from "#/lib/store-tab/fill-woolworths-trolley";

/** The bookmarklet's `href`. */
export function buildFillWoolworthsTrolleyBookmarklet(
	config: FillTrolleyConfig,
): string {
	const settings = JSON.stringify({
		apiBaseUrl: config.apiBaseUrl.replace(/\/+$/, ""),
		secret: config.secret,
	});
	const program =
		`(${fillWoolworthsTrolley.toString()})(${settings})` +
		".then(function(r){alert(r.message)}," +
		"function(e){alert('Fill trolley failed: '+e.message)});void 0";
	return `javascript:${encodeURIComponent(program)}`;
}

/** The absolute backend URL a script on another website must call. */
export function absoluteApiBaseUrl(
	apiBaseUrl: string,
	pageOrigin: string,
): string {
	return new URL(apiBaseUrl, pageOrigin).toString().replace(/\/+$/, "");
}
