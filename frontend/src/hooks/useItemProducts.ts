import { useCallback, useEffect, useState } from "react";

import { api, type ItemProducts } from "#/lib/api";

/** Where one item's product search is up to. */
export type ItemProductsState =
	| { status: "idle" }
	| { status: "loading" }
	| { status: "loaded"; data: ItemProducts }
	| { status: "failed" };

/**
 * Searches the stores for one list item while `enabled` is true — the price
 * comparison is open — and not before, so the stores are only asked when
 * somebody is looking.
 *
 * The search runs again when `itemId` changes, when `searchKey` changes (pass
 * the item's name, quantity and chips, so an edit refreshes the prices) and
 * when `retry` is called. A search that finishes after the comparison was
 * closed or re-run is ignored.
 */
export function useItemProducts(
	itemId: string,
	enabled: boolean,
	searchKey: string,
): { state: ItemProductsState; retry: () => void } {
	const [state, setState] = useState<ItemProductsState>({ status: "idle" });
	const [attempt, setAttempt] = useState(0);

	// biome-ignore lint/correctness/useExhaustiveDependencies: searchKey and attempt are deliberate re-run triggers.
	useEffect(() => {
		if (!enabled) return;
		const controller = new AbortController();
		setState({ status: "loading" });
		api.products
			.forItem(itemId, { signal: controller.signal })
			.then((data) => setState({ status: "loaded", data }))
			.catch(() => {
				if (!controller.signal.aborted) setState({ status: "failed" });
			});
		return () => controller.abort();
	}, [itemId, enabled, searchKey, attempt]);

	const retry = useCallback(() => setAttempt((n) => n + 1), []);
	return { state, retry };
}
