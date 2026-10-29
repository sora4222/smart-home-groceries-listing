import { useCallback, useEffect, useState } from "react";

import { api, type GroceryItem, type ItemProducts } from "#/lib/api";

/** Where one item's product search is up to. */
export type ItemProductsState =
	| { status: "loading" }
	| { status: "loaded"; data: ItemProducts }
	| { status: "failed" };

/**
 * Searches the stores for one list item.
 *
 * The search runs again when the item's name, quantity or chips change (so
 * an edit refreshes the prices) and when `retry` is called. A search that
 * finishes after a newer one started is ignored.
 */
export function useItemProducts(item: GroceryItem): {
	state: ItemProductsState;
	retry: () => void;
} {
	const [state, setState] = useState<ItemProductsState>({ status: "loading" });
	const [attempt, setAttempt] = useState(0);
	const searchKey = `${item.name}|${item.quantity}|${item.filter_terms.join(",")}`;

	// biome-ignore lint/correctness/useExhaustiveDependencies: searchKey and attempt are deliberate re-run triggers.
	useEffect(() => {
		const controller = new AbortController();
		setState({ status: "loading" });
		api.products
			.forItem(item.id, { signal: controller.signal })
			.then((data) => setState({ status: "loaded", data }))
			.catch(() => {
				if (!controller.signal.aborted) setState({ status: "failed" });
			});
		return () => controller.abort();
	}, [item.id, searchKey, attempt]);

	const retry = useCallback(() => setAttempt((n) => n + 1), []);
	return { state, retry };
}
