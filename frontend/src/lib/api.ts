/**
 * Typed fetch wrappers for the Rust/Axum backend. Keep this the only place
 * that knows the backend's URL shape — components call these, never
 * `fetch` directly.
 *
 * Paths, field names, status codes and the `{ detail }` error envelope are
 * unchanged from the previous FastAPI backend, so nothing else in the app
 * needed touching when it was rewritten.
 */
import { API_BASE_URL } from "#/lib/config";
import { getAuthToken } from "#/lib/auth";

export type VoiceRequestStatus = "pending" | "accepted" | "rejected";
/** Which intake channel delivered an item. */
export type IntakeSource = "webhook" | "alexa";
export type GroceryItemStatus = "pending" | "active" | "ordered";
export type GroceryItemSource = "voice" | "manual";

export interface VoiceRequest {
	id: string;
	source: IntakeSource;
	raw_text: string;
	parsed_name: string;
	parsed_quantity: number;
	status: VoiceRequestStatus;
	created_at: string;
}

export interface GroceryItem {
	id: string;
	name: string;
	quantity: number;
	status: GroceryItemStatus;
	source: GroceryItemSource;
	added_by_user_id: string | null;
	created_at: string;
}

export interface DuplicateItemDetail {
	existing_item: { id: string; name: string; quantity: number };
	message: string;
}

export class ApiError extends Error {
	constructor(
		public status: number,
		public body: unknown,
	) {
		super(typeof body === "string" ? body : JSON.stringify(body));
	}
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const token = await getAuthToken();
	const headers = new Headers(init?.headers);
	headers.set("Content-Type", "application/json");
	if (token) headers.set("Authorization", `Bearer ${token}`);

	const res = await fetch(`${API_BASE_URL}${path}`, { ...init, headers });
	if (!res.ok) {
		let body: unknown;
		try {
			body = await res.json();
		} catch {
			body = await res.text();
		}
		throw new ApiError(res.status, body);
	}
	if (res.status === 204) return undefined as T;
	return res.json() as Promise<T>;
}

export const api = {
	voice: {
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
	},
	grocery: {
		listActive: () => request<GroceryItem[]>("/api/grocery-items"),
	},
};
