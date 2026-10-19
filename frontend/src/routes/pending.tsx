import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

import { DuplicatePrompt } from "#/components/grocery/duplicate-prompt";
import { PendingRequestCard } from "#/components/pending/pending-request-card";
import { acceptFailure } from "#/lib/accept-failure";
import {
	api,
	type DuplicateItemDetail,
	duplicateDetail,
	type OnDuplicate,
	type VoiceRequest,
} from "#/lib/api";
import { ruleTermsDescription } from "#/lib/rule-terms";

export const Route = createFileRoute("/pending")({
	loader: () => api.voice.pending(),
	component: PendingPage,
});

/** Requests the user just rejected — kept visible, dulled, for undo. */
type RejectedIds = ReadonlySet<string>;

/** An accept the backend paused because the name is already on the list. */
interface HeldAccept {
	id: string;
	name: string;
	quantity: number;
	detail: DuplicateItemDetail;
}

function PendingPage() {
	const loaderData = Route.useLoaderData();
	const router = useRouter();
	const [requests, setRequests] = useState<VoiceRequest[]>(loaderData);
	const [rejectedIds, setRejectedIds] = useState<RejectedIds>(new Set());
	const [held, setHeld] = useState<HeldAccept | null>(null);

	function dropCard(id: string) {
		setRequests((prev) => prev.filter((r) => r.id !== id));
	}

	async function handleAccept(
		id: string,
		name: string,
		quantity: number,
		onDuplicate: OnDuplicate = "ask",
	) {
		let accepted: Awaited<ReturnType<typeof api.voice.accept>>;
		try {
			accepted = await api.voice.accept(id, { name, quantity }, onDuplicate);
		} catch (error) {
			const detail = duplicateDetail(error);
			if (detail) {
				// The name is already on the list; let the household decide.
				setHeld({ id, name, quantity, detail });
				return;
			}
			const failure = acceptFailure(error, name);
			if (failure.gone) dropCard(id);
			toast.error(failure.message);
			return;
		}

		setHeld(null);
		dropCard(id);
		setRejectedIds((prev) => {
			const next = new Set(prev);
			next.delete(id);
			return next;
		});
		// A voice item brings no chips of its own, so every one came from a
		// rule. A merge returns the existing item, whose chips no rule added.
		toast.success(`${name} added to the grocery list`, {
			description: ruleTermsDescription(
				onDuplicate === "merge" ? [] : accepted.grocery_item.filter_terms,
			),
		});
		try {
			await router.invalidate();
		} catch {
			// The accept succeeded; only the refresh failed (an invalidation
			// still in flight when the user navigates away rejects). Unawaited,
			// that surfaced as an unhandled "Failed to fetch" in the console.
		}
	}

	async function handleReject(id: string) {
		try {
			await api.voice.reject(id);
			setRejectedIds((prev) => new Set(prev).add(id));
		} catch {
			toast.error("Could not reject that request — try again.");
		}
	}

	if (requests.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				No pending voice requests. Try "Hey Google, add milk to the shopping
				list."
			</p>
		);
	}

	return (
		<div className="flex flex-col gap-3">
			<h1 className="text-lg font-semibold">Pending Requests</h1>
			{requests.map((request) => (
				<div key={request.id} className="flex flex-col gap-2">
					<PendingRequestCard
						request={request}
						dulled={rejectedIds.has(request.id)}
						onAccept={handleAccept}
						onReject={handleReject}
						onEdit={(id) =>
							// The held question was about the old name and quantity.
							setHeld((prev) => (prev?.id === id ? null : prev))
						}
					/>
					{held?.id === request.id && (
						<DuplicatePrompt
							detail={held.detail}
							onMerge={() =>
								handleAccept(held.id, held.name, held.quantity, "merge")
							}
							onSeparate={() =>
								handleAccept(held.id, held.name, held.quantity, "separate")
							}
							onCancel={() => setHeld(null)}
						/>
					)}
				</div>
			))}
		</div>
	);
}
