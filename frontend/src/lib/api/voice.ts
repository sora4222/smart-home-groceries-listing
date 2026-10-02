/** The intake confirmation queue: `/api/voice-requests`. */
import { request } from "#/lib/api/client";
import type { GroceryItem } from "#/lib/api/grocery";

export type VoiceRequestStatus = "pending" | "accepted" | "rejected";
/** Which intake channel delivered an item. */
export type IntakeSource = "webhook" | "alexa";

/**
 * Which queue the LLM triage step put a request in. `approved` and `skipped`
 * are in Pending Requests; `held` and `rejected` are in the Triage view;
 * `unchecked` is waiting for the classifier and shown nowhere.
 */
export type TriageStatus =
	| "unchecked"
	| "approved"
	| "rejected"
	| "held"
	| "skipped";

export interface VoiceRequest {
	id: string;
	source: IntakeSource;
	raw_text: string;
	parsed_name: string;
	parsed_quantity: number;
	status: VoiceRequestStatus;
	created_at: string;
	triage_status: TriageStatus;
	/** The classifier's one-sentence reason, or why it could not answer. */
	triage_reason: string | null;
	/** 0 to 1, when the classifier answered. */
	triage_confidence: number | null;
}

export const voiceApi = {
	pending: () => request<VoiceRequest[]>("/api/voice-requests"),
	accept: (
		id: string,
		body?: { name?: string; quantity?: number },
		merge = false,
	) =>
		request<{ voice_request: VoiceRequest; grocery_item: GroceryItem }>(
			`/api/voice-requests/${id}/accept${merge ? "?merge=true" : ""}`,
			{ method: "POST", body: JSON.stringify(body ?? {}) },
		),
	reject: (id: string) =>
		request<VoiceRequest>(`/api/voice-requests/${id}/reject`, {
			method: "POST",
		}),
};
