import { PendingRequestCard } from "#/components/pending/pending-request-card";
import type { VoiceRequest } from "#/lib/api";

/**
 * The top half of Pending: the requests waiting for a decision, or
 * "No items pending" when there are none.
 */
export function PendingSection({
	requests,
	rejectedIds,
	onAccept,
	onReject,
}: {
	requests: VoiceRequest[];
	rejectedIds: ReadonlySet<string>;
	onAccept: (id: string, name: string, quantity: number) => void;
	onReject: (id: string) => void;
}) {
	return (
		<section aria-label="Pending requests" className="flex flex-col gap-3">
			{requests.length === 0 ? (
				<p className="rounded-md border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
					No items pending
				</p>
			) : (
				requests.map((request) => (
					<PendingRequestCard
						key={request.id}
						request={request}
						dulled={rejectedIds.has(request.id)}
						onAccept={onAccept}
						onReject={onReject}
					/>
				))
			)}
		</section>
	);
}
