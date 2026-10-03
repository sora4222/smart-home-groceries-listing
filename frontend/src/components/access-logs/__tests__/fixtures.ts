import type { AccessLogEntry } from "#/lib/api";

/** A logged request, with every field a row shows. */
export function logEntry(
	overrides: Partial<AccessLogEntry> = {},
): AccessLogEntry {
	return {
		id: 7,
		occurred_at: "2026-10-02T04:40:16Z",
		source_ip: "192.168.1.20",
		forwarded_for: null,
		user_id: "user_abc",
		method: "GET",
		path: "/api/grocery-items",
		status_code: 200,
		duration_ms: 4,
		...overrides,
	};
}
