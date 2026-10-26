import { TriageList } from "#/components/triage/triage-list";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import type { TriageTab, VoiceRequest } from "#/lib/api";

/**
 * The bottom half of Pending: items the checker held or rejected, in two
 * tabs. Accept moves an item up into the pending list.
 */
export function TriageSection({
	held,
	rejected,
	tab,
	busyId,
	onAccept,
	onReject,
	onRestore,
}: {
	held: VoiceRequest[];
	rejected: VoiceRequest[];
	tab: TriageTab;
	busyId: string | null;
	onAccept: (request: VoiceRequest) => void;
	onReject: (request: VoiceRequest) => void;
	onRestore: (request: VoiceRequest) => void;
}) {
	const actions = { busyId, onAccept, onReject, onRestore };
	return (
		<section aria-labelledby="triage-heading" className="flex flex-col gap-2">
			<h2 id="triage-heading" className="text-base font-semibold">
				Triage
			</h2>
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
						emptyText="Nothing held for review."
						{...actions}
					/>
				</TabsContent>
				<TabsContent value="rejected">
					<TriageList
						requests={rejected}
						emptyText="Nothing rejected."
						{...actions}
					/>
				</TabsContent>
			</Tabs>
		</section>
	);
}
