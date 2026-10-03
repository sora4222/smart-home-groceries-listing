/**
 * How one access-log row reads on the `/logs` page: the time, who it was,
 * and how the status code is coloured. Pure, so it is unit-tested.
 */
import type { AccessLogEntry } from "#/lib/api/access-logs";

/** The user id the backend records when nobody was signed in. */
export const UNAUTHENTICATED = "unauthenticated";

/** The badge look for a status code. */
export type StatusTone = "secondary" | "outline" | "destructive";

/** Local date and time, to the second: "2 Oct, 14:40:16". */
export function formatLoggedAt(iso: string, locale?: string): string {
	return new Date(iso).toLocaleString(locale, {
		day: "numeric",
		month: "short",
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hour12: false,
	});
}

/** Who made the request, in words. */
export function describeUser(userId: string): string {
	return userId === UNAUTHENTICATED ? "Not signed in" : userId;
}

/** Where the request came from: the connection, plus any forwarded claim. */
export function describeSource(
	entry: Pick<AccessLogEntry, "source_ip" | "forwarded_for">,
): string {
	const source = entry.source_ip ?? "unknown";
	return entry.forwarded_for
		? `${source} (says ${entry.forwarded_for})`
		: source;
}

/** Success is quiet, a refusal is outlined, a server failure is red. */
export function statusTone(status: number): StatusTone {
	if (status >= 500) return "destructive";
	if (status >= 400) return "outline";
	return "secondary";
}
