import { useEffect, useRef } from "react";

import { type CountEventType, connect, subscribe } from "#/lib/ws";

/**
 * Calls `onRise` when a pushed count of `eventType` goes up — a new item, not
 * a decision that lowered it. The first event only sets the baseline, so
 * opening the app never toasts for items that were already waiting.
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
			if (event.type !== eventType) return;
			if (
				previousCount.current !== null &&
				event.count > previousCount.current
			) {
				latestOnRise.current();
			}
			previousCount.current = event.count;
		});
	}, [eventType]);
}
