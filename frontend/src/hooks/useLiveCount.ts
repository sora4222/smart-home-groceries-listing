import { useEffect, useState } from "react";

import { type CountEventType, connect, subscribe } from "#/lib/ws";

/**
 * A count the backend keeps fresh: loaded once with `load`, then replaced by
 * every pushed event of `eventType` — no polling. Pass a `load` defined at
 * module level, so the effect does not re-run on every render.
 *
 * A failed load leaves the count at 0 rather than blocking the page.
 */
export function useLiveCount(
	eventType: CountEventType,
	load: () => Promise<number>,
): number {
	const [count, setCount] = useState(0);

	useEffect(() => {
		let cancelled = false;
		load()
			.then((initial) => {
				if (!cancelled) setCount(initial);
			})
			.catch(() => {
				// A nav badge is not worth an error on every page.
			});

		connect();
		const unsubscribe = subscribe((event) => {
			if (event.type === eventType) setCount(event.count);
		});

		return () => {
			cancelled = true;
			unsubscribe();
		};
	}, [eventType, load]);

	return count;
}
