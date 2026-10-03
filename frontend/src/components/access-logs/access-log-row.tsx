import { Badge } from "#/components/ui/badge";
import {
	describeSource,
	describeUser,
	formatLoggedAt,
	statusTone,
} from "#/lib/access-log-display";
import type { AccessLogEntry } from "#/lib/api";

/** One answered request: when, who, from where, what, and the answer. */
export function AccessLogRow({ entry }: { entry: AccessLogEntry }) {
	return (
		<li
			data-log-path={entry.path}
			className="flex flex-col gap-1 border-b border-border py-2 text-sm last:border-b-0"
		>
			<div className="flex items-center gap-2">
				<Badge variant={statusTone(entry.status_code)}>
					{entry.status_code}
				</Badge>
				<span className="font-mono font-medium">{entry.method}</span>
				<span className="min-w-0 break-all font-mono">{entry.path}</span>
			</div>
			<div className="flex flex-wrap gap-x-3 text-xs text-muted-foreground">
				<time dateTime={entry.occurred_at}>
					{formatLoggedAt(entry.occurred_at)}
				</time>
				<span>{describeUser(entry.user_id)}</span>
				<span>{describeSource(entry)}</span>
				<span>{entry.duration_ms} ms</span>
			</div>
		</li>
	);
}
