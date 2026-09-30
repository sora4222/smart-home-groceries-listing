/**
 * The one function that talks to the backend: adds the session token, sends
 * JSON, and turns a non-2xx answer into an {@link ApiError}.
 *
 * Errors arrive as `{ "detail": ... }` — a string for most failures, an
 * object for the duplicate-item 409 — and are kept whole on `ApiError.body`.
 */
import { getAuthToken } from "#/lib/auth";
import { API_BASE_URL } from "#/lib/config";

/** A non-2xx answer from the backend, with its parsed body. */
export class ApiError extends Error {
	constructor(
		public status: number,
		public body: unknown,
	) {
		super(typeof body === "string" ? body : JSON.stringify(body));
	}
}

/** Sends a request to the backend and parses the JSON answer. */
export async function request<T>(path: string, init?: RequestInit): Promise<T> {
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
