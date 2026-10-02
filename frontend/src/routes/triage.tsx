import { createFileRoute } from "@tanstack/react-router";

import { TriageList } from "#/components/triage/triage-list";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useTriageDecisions } from "#/hooks/useTriageDecisions";
import { api, type TriageTab } from "#/lib/api";

export const Route = createFileRoute("/triage")({
	validateSearch: (search: Record<string, unknown>): { tab?: TriageTab } =>
		search.tab === "rejected" ? { tab: "rejected" } : {},
	loader: async () => {
		const [held, rejected] = await Promise.all([
			api.triage.list("held"),
			api.triage.list("rejected"),
		]);
		return { held, rejected };
	},
	component: TriagePage,
});

/**
 * `/triage` — intake requests the LLM triage step did not pass straight to
 * Pending Requests (spec: "Triage View"). Held for review needs a person;
 * Rejected is the checker saying "not a supermarket item".
 */
function TriagePage() {
	const { tab = "held" } = Route.useSearch();
	const { held, rejected, busyId, accept, reject } = useTriageDecisions(
		Route.useLoaderData(),
	);

	return (
		<div className="flex flex-col gap-3">
			<h1 className="text-lg font-semibold">Triage</h1>
			<p className="text-sm text-muted-foreground">
				Items the checker was not sure about, or thinks a supermarket does not
				sell. Accept sends an item to Pending Requests.
			</p>
			<Tabs defaultValue={tab}>
				<TabsList className="w-full sm:w-fit">
					<TabsTrigger value="held">
						Held for review ({held.length})
					</TabsTrigger>
					<TabsTrigger value="rejected">
						Rejected ({rejected.length})
					</TabsTrigger>
				</TabsList>
				<TabsContent value="held">
					<TriageList
						requests={held}
						emptyText="Nothing is held for review."
						busyId={busyId}
						onAccept={accept}
						onReject={reject}
					/>
				</TabsContent>
				<TabsContent value="rejected">
					<TriageList
						requests={rejected}
						emptyText="Nothing was rejected."
						busyId={busyId}
						onAccept={accept}
						onReject={reject}
					/>
				</TabsContent>
			</Tabs>
		</div>
	);
}
