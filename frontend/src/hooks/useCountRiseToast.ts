import { useEffect, useRef } from "react";

import { type CountEventType, connect, countIn, subscribe } from "#/lib/ws";

/**
 * Calls `onRise` when a pushed count of `eventType` goes up — a new item, not
 * a decision that lowered it. Connection snapshots silently reset the baseline.
 * Without a snapshot, the first count is a baseline for older servers.
 */
export function useCountRiseToast(
	eventType: CountEventType,
	onRise: () => void,
): void {
	const previousCount = useRef<number | null>(null);
	const latestOnRise = useRef(onRise);
	latestOnRise.current = onRise;

	useEffect(() => {
		connect();
		return subscribe((event) => {
			const count = countIn(event, eventType);
			if (count === null) return;
			const rose =
				event.type !== "counts_snapshot" &&
				previousCount.current !== null &&
				count > previousCount.current;
			previousCount.current = count;
			if (rose) latestOnRise.current();
		});
	}, [eventType]);
}
