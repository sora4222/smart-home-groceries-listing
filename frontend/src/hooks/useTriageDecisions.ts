import { useRouter } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

import { api, type VoiceRequest } from "#/lib/api";

/** Both Triage tabs, as loaded. */
export interface TriageLists {
	held: VoiceRequest[];
	rejected: VoiceRequest[];
}

/**
 * The triage lists on the Pending page and their actions: accept (up into
 * the pending list), reject, and put a Google Tasks item back on its list.
 * A decided card leaves its list as soon as the backend agrees; a failure
 * leaves it in place with a toast.
 */
export function useTriageDecisions(initial: TriageLists) {
	const router = useRouter();
	const [lists, setLists] = useState<TriageLists>(initial);
	const [busyId, setBusyId] = useState<string | null>(null);

	function remove(id: string) {
		setLists((prev) => ({
			held: prev.held.filter((r) => r.id !== id),
			rejected: prev.rejected.filter((r) => r.id !== id),
		}));
	}

	async function decide(
		request: VoiceRequest,
		call: (id: string) => Promise<VoiceRequest>,
		done: () => void,
		failure: string,
	) {
		setBusyId(request.id);
		try {
			await call(request.id);
			remove(request.id);
			done();
		} catch {
			toast.error(failure);
		} finally {
			setBusyId(null);
		}
	}

	const accept = (request: VoiceRequest) =>
		decide(
			request,
			api.triage.accept,
			() => {
				toast.success(`${request.parsed_name} moved to pending`);
				// Reload so it shows in the pending list above.
				router.invalidate().catch(() => {});
			},
			"Could not move that request — try again.",
		);

	const reject = (request: VoiceRequest) =>
		decide(
			request,
			api.triage.reject,
			() => toast(`${request.parsed_name} rejected`),
			"Could not reject that request — try again.",
		);

	const restore = (request: VoiceRequest) =>
		decide(
			request,
			api.triage.restore,
			() =>
				toast.success(
					`${request.parsed_name} put back in Google Tasks. It comes back on the next check.`,
				),
			"Could not put that back in Google Tasks — try again.",
		);

	return { ...lists, busyId, accept, reject, restore };
}
