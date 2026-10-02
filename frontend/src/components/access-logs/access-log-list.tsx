import { AccessLogRow } from "#/components/access-logs/access-log-row";
import type { AccessLogEntry } from "#/lib/api";

/** Every loaded row, newest first, or a line saying there are none. */
export function AccessLogList({ entries }: { entries: AccessLogEntry[] }) {
	if (entries.length === 0) {
		return (
			<p className="py-4 text-sm text-muted-foreground">
				No requests logged yet.
			</p>
		);
	}
	return (
		<ul aria-label="Access log" className="flex flex-col">
			{entries.map((entry) => (
				<AccessLogRow key={entry.id} entry={entry} />
			))}
		</ul>
	);
}
