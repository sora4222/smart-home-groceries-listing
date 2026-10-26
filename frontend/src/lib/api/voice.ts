/** The intake confirmation queue: `/api/voice-requests`. */
import { request } from "#/lib/api/client";
import type { GroceryItem, OnDuplicate } from "#/lib/api/grocery";

export type VoiceRequestStatus = "pending" | "accepted" | "rejected";
/** Which intake channel delivered an item. */
export type IntakeSource = "webhook" | "alexa" | "tasks";

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

/** What an accept did: the list item, and whether it added to one already there. */
export interface VoiceAcceptResult {
	voice_request: VoiceRequest;
	grocery_item: GroceryItem;
	merged: boolean;
}

export const voiceApi = {
	pending: () => request<VoiceRequest[]>("/api/voice-requests"),
	/**
	 * Accepts a request onto the list. `ask` — the default — makes the backend
	 * answer 409 when the name is already on the list, so the user can choose.
	 */
	accept: (
		id: string,
		body?: { name?: string; quantity?: number },
		onDuplicate: OnDuplicate = "ask",
	) =>
		request<VoiceAcceptResult>(
			`/api/voice-requests/${id}/accept?on_duplicate=${onDuplicate}`,
			{ method: "POST", body: JSON.stringify(body ?? {}) },
		),
	/** Takes an accepted item back off the list and returns it to the queue. */
	undoAccept: (id: string) =>
		request<VoiceRequest>(`/api/voice-requests/${id}/undo-accept`, {
			method: "POST",
		}),
	reject: (id: string) =>
		request<VoiceRequest>(`/api/voice-requests/${id}/reject`, {
			method: "POST",
		}),
};
