import type { VoiceRequest } from "#/lib/api";

/** A request triage held, with every field a card shows. */
export function heldRequest(
	overrides: Partial<VoiceRequest> = {},
): VoiceRequest {
	return {
		id: "22222222-2222-2222-2222-222222222222",
		source: "alexa",
		raw_text: "add flibber to the list",
		parsed_name: "flibber",
		parsed_quantity: 1,
		status: "pending",
		created_at: "2026-10-02T03:00:00Z",
		triage_status: "held",
		triage_reason: "Not sure what this is.",
		triage_confidence: 0.4,
		...overrides,
	};
}
