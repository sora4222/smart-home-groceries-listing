/** The intake confirmation queue: `/api/voice-requests`. */
import { request } from "#/lib/api/client";
import type { GroceryItem } from "#/lib/api/grocery";

export type VoiceRequestStatus = "pending" | "accepted" | "rejected";
/** Which intake channel delivered an item. */
export type IntakeSource = "webhook" | "alexa";

export interface VoiceRequest {
	id: string;
	source: IntakeSource;
	raw_text: string;
	parsed_name: string;
	parsed_quantity: number;
	status: VoiceRequestStatus;
	created_at: string;
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
