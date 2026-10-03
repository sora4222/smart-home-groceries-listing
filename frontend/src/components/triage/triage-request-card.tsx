import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import type { VoiceRequest } from "#/lib/api";
import { intakeSourceLabel } from "#/lib/intake-source";
import { confidenceLabel } from "#/lib/triage-confidence";

interface TriageRequestCardProps {
	request: VoiceRequest;
	/** True while this card's Accept or Reject is being saved. */
	busy: boolean;
	onAccept: (request: VoiceRequest) => void;
	onReject: (request: VoiceRequest) => void;
	/** Puts a Google Tasks item back on its list. */
	onRestore: (request: VoiceRequest) => void;
}

/**
 * One request the triage step held or rejected (spec: "Triage View"). Shows
 * what was heard, where it came from, what the checker said and when. Accept
 * moves it on to Pending Requests — a person still accepts it onto the list
 * there. Reject confirms it is not wanted. A Google Tasks item can also be
 * put back on its list ("Restore to source"), so the request is not lost.
 */
export function TriageRequestCard({
	request,
	busy,
	onAccept,
	onReject,
	onRestore,
}: TriageRequestCardProps) {
	const name = request.parsed_name;
	const sure = confidenceLabel(request.triage_confidence);

	return (
		<Card data-request-name={name}>
			<CardHeader className="flex-row flex-wrap items-start justify-between gap-2">
				<CardTitle className="text-base">{name}</CardTitle>
				<div className="flex gap-1">
					<Badge variant="outline">{intakeSourceLabel(request.source)}</Badge>
					<Badge
						variant={request.triage_status === "held" ? "secondary" : "outline"}
					>
						{request.triage_status === "held" ? "Held for review" : "Rejected"}
					</Badge>
				</div>
			</CardHeader>
			<CardContent className="flex flex-col gap-1 text-sm">
				{request.raw_text !== name && (
					<p className="text-muted-foreground">Heard: “{request.raw_text}”</p>
				)}
				<p>
					{request.triage_reason ?? "No reason given."}
					{sure && <span className="text-muted-foreground"> ({sure})</span>}
				</p>
				<p className="text-xs text-muted-foreground">
					{new Date(request.created_at).toLocaleString()}
				</p>
			</CardContent>
			<CardFooter className="flex-wrap gap-2">
				<Button
					size="sm"
					className="flex-1 sm:flex-none"
					disabled={busy}
					onClick={() => onAccept(request)}
					aria-label={`Accept ${name}`}
				>
					Accept
				</Button>
				<Button
					size="sm"
					variant="outline"
					className="flex-1 sm:flex-none"
					disabled={busy}
					onClick={() => onReject(request)}
					aria-label={`Reject ${name}`}
				>
					Reject
				</Button>
				{request.source === "tasks" && (
					<Button
						size="sm"
						variant="ghost"
						className="w-full sm:w-auto"
						disabled={busy}
						onClick={() => onRestore(request)}
						aria-label={`Put ${name} back in Google Tasks`}
					>
						Put back in Google Tasks
					</Button>
				)}
			</CardFooter>
		</Card>
	);
}
