/**
 * The WebSocket address for one connection attempt. Browsers cannot set
 * headers on a WebSocket, so the session token rides in `?token=`; the
 * backend reads it there for `/ws` only. Pure, so it is unit-tested.
 */
export function socketUrl(base: string, token: string | null): string {
	if (!token) return base;
	const separator = base.includes("?") ? "&" : "?";
	return `${base}${separator}token=${encodeURIComponent(token)}`;
}
