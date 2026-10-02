import { TriageRequestCard } from "#/components/triage/triage-request-card";
import type { VoiceRequest } from "#/lib/api";

interface TriageListProps {
	requests: VoiceRequest[];
	/** What to say when the tab is empty. */
	emptyText: string;
	busyId: string | null;
	onAccept: (request: VoiceRequest) => void;
	onReject: (request: VoiceRequest) => void;
}

/** The cards of one Triage tab, or a line saying there are none. */
export function TriageList({
	requests,
	emptyText,
	busyId,
	onAccept,
	onReject,
}: TriageListProps) {
	if (requests.length === 0) {
		return <p className="py-4 text-sm text-muted-foreground">{emptyText}</p>;
	}
	return (
		<div className="flex flex-col gap-3">
			{requests.map((request) => (
				<TriageRequestCard
					key={request.id}
					request={request}
					busy={busyId === request.id}
					onAccept={onAccept}
					onReject={onReject}
				/>
			))}
		</div>
	);
}
