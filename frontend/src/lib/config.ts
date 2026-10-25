/**
 * Browser addresses come from VITE_* settings. Server rendering can use
 * SERVER_API_BASE_URL for an internal network. Rules: lib/backend-url.ts.
 */
import { resolveBackendUrls, resolveServerApiUrl } from "#/lib/backend-url";
import { publicConfig } from "#/lib/public-config";

const urls = resolveBackendUrls(
	publicConfig(),
	import.meta.env.SSR ? undefined : window.location.origin,
);

export const API_BASE_URL: string = import.meta.env.SSR
	? resolveServerApiUrl(urls.apiBaseUrl, process.env.SERVER_API_BASE_URL)
	: urls.apiBaseUrl;

export const WS_URL: string = urls.wsUrl;
