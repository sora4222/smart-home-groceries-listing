/**
 * WebSocket client for real-time push events (currently just
 * `voice_request_added`). Auto-reconnects with backoff; safe to call
 * `connect` once at app mount — it is a no-op if a socket is already open.
 */
import { WS_URL } from "#/lib/config";

export interface VoiceRequestAddedEvent {
	type: "voice_request_added";
	count: number;
}

type ServerEvent = VoiceRequestAddedEvent;

type Listener = (event: ServerEvent) => void;

const listeners = new Set<Listener>();
let socket: WebSocket | null = null;
let retryDelayMs = 1000;
let retryTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleReconnect() {
	if (retryTimer) return;
	retryTimer = setTimeout(() => {
		retryTimer = null;
		connect();
	}, retryDelayMs);
	retryDelayMs = Math.min(retryDelayMs * 2, 15000);
}

export function connect(): void {
	if (typeof window === "undefined") return; // SSR guard — client only
	if (
		socket &&
		(socket.readyState === WebSocket.OPEN ||
			socket.readyState === WebSocket.CONNECTING)
	) {
		return;
	}

	socket = new WebSocket(WS_URL);

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
