import { useEffect, useState } from "react";

import { api } from "#/lib/api";
import { connect, subscribe } from "#/lib/ws";

/**
 * Tracks the pending-voice-request count for the nav badge, kept in sync by
 * an initial fetch plus the `voice_request_added` WebSocket push — no
 * polling.
 */
export function usePendingCount(): number {
	const [count, setCount] = useState(0);

	useEffect(() => {
		let cancelled = false;
		api.voice
			.pending()
			.then((items) => {
				if (!cancelled) setCount(items.length);
			})
			.catch(() => {
				// Nav badge degrades to 0 rather than blocking the page on an error.
			});

		connect();
		const unsubscribe = subscribe((event) => {
			if (event.type === "voice_request_added") setCount(event.count);
		});

		return () => {
			cancelled = true;
			unsubscribe();
		};
	}, []);

	return count;
}
