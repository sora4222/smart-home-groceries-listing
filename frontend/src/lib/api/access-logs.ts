/**
 * The access log: `/api/access-logs`. Every request the backend answered,
 * newest first, for the `/logs` page (`docs/features/FEATURE_ACCESS_LOGS.md`).
 */
import { request } from "#/lib/api/client";

/** One answered request. */
export interface AccessLogEntry {
	id: number;
	occurred_at: string;
	/** The address that opened the connection; null when unknown. */
	source_ip: string | null;
	/** What a forwarding header claimed. Anyone can send it — not proof. */
	forwarded_for: string | null;
	/** The signed-in user's id, or `"unauthenticated"`. */
	user_id: string;
	method: string;
	path: string;
	status_code: number;
	duration_ms: number;
}

/** One page, newest first. `next_before` asks for the next, older page. */
export interface AccessLogPage {
	entries: AccessLogEntry[];
	next_before: number | null;
}

/** Which page to ask for. */
export interface AccessLogPageQuery {
	before?: number;
	hideHealthChecks: boolean;
}

/** The query string for one page. */
export function accessLogQuery({
	before,
	hideHealthChecks,
}: AccessLogPageQuery): string {
	const params = new URLSearchParams({
		hide_health_checks: String(hideHealthChecks),
	});
	if (before !== undefined) params.set("before", String(before));
	return params.toString();
}

export const accessLogsApi = {
	list: (query: AccessLogPageQuery) =>
		request<AccessLogPage>(`/api/access-logs?${accessLogQuery(query)}`),
};
