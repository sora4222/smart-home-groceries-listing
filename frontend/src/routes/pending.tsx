import { useState } from "react";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { PendingRequestCard } from "#/components/pending/pending-request-card";
import { api, type VoiceRequest } from "#/lib/api";

export const Route = createFileRoute("/pending")({
	loader: () => api.voice.pending(),
	component: PendingPage,
});

/** Requests the user just rejected — kept visible, dulled, for undo. */
type RejectedIds = ReadonlySet<string>;

function PendingPage() {
	const loaderData = Route.useLoaderData();
	const router = useRouter();
	const [requests, setRequests] = useState<VoiceRequest[]>(loaderData);
	const [rejectedIds, setRejectedIds] = useState<RejectedIds>(new Set());

	async function handleAccept(id: string, name: string, quantity: number) {
		try {
			await api.voice.accept(id, { name, quantity });
			setRequests((prev) => prev.filter((r) => r.id !== id));
			setRejectedIds((prev) => {
				const next = new Set(prev);
				next.delete(id);
				return next;
			});
			toast.success(`${name} added to the grocery list`);
			router.invalidate();
		} catch {
			toast.error("Could not accept that request — try again.");
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
				<PendingRequestCard
					key={request.id}
					request={request}
					dulled={rejectedIds.has(request.id)}
					onAccept={handleAccept}
					onReject={handleReject}
				/>
			))}
		</div>
	);
}
