import { createFileRoute } from "@tanstack/react-router";

import { PendingSection } from "#/components/pending/pending-section";
import { TriageSection } from "#/components/triage/triage-section";
import { usePendingDecisions } from "#/hooks/usePendingDecisions";
import { useTriageDecisions } from "#/hooks/useTriageDecisions";
import { api, type TriageTab } from "#/lib/api";

export const Route = createFileRoute("/pending")({
	validateSearch: (search: Record<string, unknown>): { tab?: TriageTab } =>
		search.tab === "rejected" ? { tab: "rejected" } : {},
	loader: async () => {
		const [pending, held, rejected] = await Promise.all([
			api.voice.pending(),
			api.triage.list("held"),
			api.triage.list("rejected"),
		]);
		return { pending, held, rejected };
	},
	component: PendingPage,
});

/**
 * `/pending` — requests waiting for a decision on top, and below them the
 * items the checker held or rejected (`?tab=rejected` opens that tab).
 */
function PendingPage() {
	const { pending, held, rejected } = Route.useLoaderData();
	const { tab = "held" } = Route.useSearch();
	const decisions = usePendingDecisions(pending);
	const triage = useTriageDecisions({ held, rejected });

	return (
		<div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
			<h1 className="text-lg font-semibold">Pending</h1>
			<PendingSection
				requests={decisions.requests}
				rejectedIds={decisions.rejectedIds}
				onAccept={decisions.accept}
				onReject={decisions.reject}
			/>
			<TriageSection
				held={triage.held}
				rejected={triage.rejected}
				tab={tab}
				busyId={triage.busyId}
				onAccept={triage.accept}
				onReject={triage.reject}
				onRestore={triage.restore}
			/>
		</div>
	);
}
