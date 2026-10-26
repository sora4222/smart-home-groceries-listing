import { useRouter } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

import { acceptFailure } from "#/lib/accept-failure";
import { acceptMessage } from "#/lib/accept-message";
import { api, type VoiceRequest } from "#/lib/api";
import { ApiError } from "#/lib/api/client";
import { ruleTermsDescription } from "#/lib/rule-terms";
import { toastWithUndo } from "#/lib/undo-toast";

/**
 * Pending Requests' cards and their actions.
 *
 * Accept always adds: an item already on the list gets the quantity added to
 * it. The toast has Undo, which takes the item back off and returns the card.
 * A rejected card stays, dulled, so accepting it again is the undo.
 *
 * The loader's list is the source of truth; only rejected cards (which the
 * backend no longer lists) are kept here.
 */
export function usePendingDecisions(pending: VoiceRequest[]) {
	const router = useRouter();
	const [rejected, setRejected] = useState<VoiceRequest[]>([]);
	const [accepting, setAccepting] = useState<ReadonlySet<string>>(new Set());

	const requests = [
		...pending,
		...rejected.filter((r) => !pending.some((p) => p.id === r.id)),
	];
	const rejectedIds = new Set(rejected.map((r) => r.id));

	function forget(id: string) {
		setRejected((prev) => prev.filter((r) => r.id !== id));
	}

	async function refresh() {
		try {
			await router.invalidate();
		} catch {
			// The change was saved; only the refresh failed (navigating away
			// mid-refresh rejects it).
		}
	}

	async function accept(id: string, name: string, quantity: number) {
		setAccepting((prev) => new Set(prev).add(id));
		try {
			const result = await api.voice.accept(id, { name, quantity }, "merge");
			forget(id);
			// A voice item brings no chips of its own, so every one came from a
			// rule. A merge keeps the existing item's chips.
			toastWithUndo(acceptMessage(name, result), {
				description: ruleTermsDescription(
					result.merged ? [] : result.grocery_item.filter_terms,
				),
				undo: () => api.voice.undoAccept(id),
				failure: (error) =>
					error instanceof ApiError && error.status === 409
						? "The list is ready to order. Release it to undo."
						: "Could not undo that. Check the list.",
				after: refresh,
			});
			await refresh();
		} catch (error) {
			const failure = acceptFailure(error, name);
			if (failure.gone) forget(id);
			toast.error(failure.message);
			if (failure.gone) await refresh();
		} finally {
			setAccepting((prev) => {
				const next = new Set(prev);
				next.delete(id);
				return next;
			});
		}
	}

	async function reject(id: string) {
		const request = requests.find((r) => r.id === id);
		try {
			await api.voice.reject(id);
			if (request) setRejected((prev) => [...prev, request]);
		} catch {
			toast.error("Could not reject that request — try again.");
		}
	}

	// A card being accepted stays, busy, so its edits survive a failure.
	return { requests, rejectedIds, busyIds: accepting, accept, reject };
}
