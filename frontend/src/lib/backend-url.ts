/**
 * Where the browser finds the backend. The web app can run on a different
 * computer from the backend (for example a laptop, with the backend behind a
 * Cloudflare Tunnel), so both addresses come from `VITE_*` values.
 *
 * Blank values count as not set. The socket address follows the API address
 * unless given (`https` → `wss`). Pure, so it is unit-tested.
 */

/** The same-machine backend, used when `VITE_API_BASE_URL` is not set. */
const DEFAULT_API_BASE_URL = "http://localhost:8000";

/** The environment values the addresses are decided from. */
export interface BackendUrlEnv {
	VITE_API_BASE_URL?: string;
	VITE_WS_URL?: string;
}

/** The backend's HTTP base address and its WebSocket address. */
export interface BackendUrls {
	apiBaseUrl: string;
	wsUrl: string;
}

/** Reads both addresses, failing loudly on one the browser cannot use. */
export function resolveBackendUrls(env: BackendUrlEnv): BackendUrls {
	const apiBaseUrl = withoutTrailingSlash(
		env.VITE_API_BASE_URL?.trim() || DEFAULT_API_BASE_URL,
	);
	requireScheme("VITE_API_BASE_URL", apiBaseUrl, ["http://", "https://"]);

	const wsUrl =
		env.VITE_WS_URL?.trim() || `${apiBaseUrl.replace(/^http/, "ws")}/ws`;
	requireScheme("VITE_WS_URL", wsUrl, ["ws://", "wss://"]);

	return { apiBaseUrl, wsUrl };
}

function withoutTrailingSlash(url: string): string {
	return url.replace(/\/+$/, "");
}

function requireScheme(key: string, url: string, schemes: string[]): void {
	if (!schemes.some((scheme) => url.startsWith(scheme))) {
		throw new Error(`${key} must start with ${schemes.join(" or ")}: "${url}"`);
	}
}
