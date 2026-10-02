/**
 * The Triage view: `/api/triage`. Requests the LLM triage step held for
 * review or rejected. "Accept" here moves a request on to Pending Requests;
 * it never puts anything on the list.
 */
import { request } from "#/lib/api/client";
import type { VoiceRequest } from "#/lib/api/voice";

/** The two tabs of the Triage view. */
export type TriageTab = "held" | "rejected";

export const triageApi = {
	list: (tab: TriageTab) => request<VoiceRequest[]>(`/api/triage?tab=${tab}`),
	accept: (id: string) =>
		request<VoiceRequest>(`/api/triage/${id}/accept`, { method: "POST" }),
	reject: (id: string) =>
		request<VoiceRequest>(`/api/triage/${id}/reject`, { method: "POST" }),
};
