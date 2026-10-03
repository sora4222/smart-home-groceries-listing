/**
 * WebSocket client for real-time push events: `voice_request_added` (the
 * Pending Requests count) and `triage_held` (the held-for-review count). Auto-reconnects with backoff; safe to call
 * `connect` once at app mount — it is a no-op if a socket is already open.
 */
import { getAuthToken } from "#/lib/auth";
import { AUTH_MODE } from "#/lib/auth-mode";
import { WS_URL } from "#/lib/config";
import { socketUrl } from "#/lib/ws-url";

export interface VoiceRequestAddedEvent {
	type: "voice_request_added";
	count: number;
}

/** The held-for-review count changed. */
export interface TriageHeldEvent {
	type: "triage_held";
	count: number;
}

/** Every event the backend pushes. Each carries a fresh count. */
export type ServerEvent = VoiceRequestAddedEvent | TriageHeldEvent;

/** The `type` of an event that carries a count. */
export type CountEventType = ServerEvent["type"];

type Listener = (event: ServerEvent) => void;

const listeners = new Set<Listener>();
let socket: WebSocket | null = null;
let retryDelayMs = 1000;
let retryTimer: ReturnType<typeof setTimeout> | null = null;
/** True while a session token is being fetched for the next socket. */
let opening = false;

function scheduleReconnect() {
	if (retryTimer) return;
	retryTimer = setTimeout(() => {
		retryTimer = null;
		connect();
	}, retryDelayMs);
	retryDelayMs = Math.min(retryDelayMs * 2, 15000);
}

/**
 * Opens the socket unless one is open, opening, or waiting to retry. With
 * sign-in on, a fresh session token is fetched for every attempt (they live
 * about a minute); signed out, it waits and tries again later.
 */
export function connect(): void {
	if (typeof window === "undefined") return; // SSR guard — client only
	if (opening || retryTimer) return;
	if (
		socket &&
		(socket.readyState === WebSocket.OPEN ||
			socket.readyState === WebSocket.CONNECTING)
	) {
		return;
	}

	opening = true;
	getAuthToken()
		.catch(() => null)
		.then((token) => {
			opening = false;
			if (AUTH_MODE === "clerk" && !token) {
				scheduleReconnect();
				return;
			}
			open(socketUrl(WS_URL, token));
		});
}

/** Opens one socket and wires its handlers. */
function open(url: string): void {
	socket = new WebSocket(url);

	socket.onopen = () => {
		retryDelayMs = 1000;
	};

	socket.onmessage = (event) => {
		try {
			const parsed = JSON.parse(event.data) as ServerEvent;
			for (const listener of listeners) listener(parsed);
		} catch {
			// Ignore malformed frames rather than crash the socket handler.
		}
	};

	socket.onclose = () => {
		socket = null;
		scheduleReconnect();
	};

	socket.onerror = () => {
		socket?.close();
	};
}

export function subscribe(listener: Listener): () => void {
	listeners.add(listener);
	return () => listeners.delete(listener);
}
