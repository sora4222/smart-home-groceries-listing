/**
 * The past purchases of the products a price comparison shows.
 *
 * Asks once per set of products and answers with a map keyed by
 * {@link productKey}. A product never bought has no entry. When the history
 * cannot be read the map stays empty: the comparison still works, it just
 * shows no "Bought N times".
 */
import { useEffect, useState } from "react";

import {
	api,
	type ProductHistory,
	type ProductRef,
	productKey,
} from "#/lib/api";

export function useProductHistory(
	products: ProductRef[],
): Map<string, ProductHistory> {
	const [history, setHistory] = useState(new Map<string, ProductHistory>());
	const key = products.map((p) => productKey(p.store, p.product_id)).join("|");

	// biome-ignore lint/correctness/useExhaustiveDependencies: `key` stands for `products`, which is a new array every render.
	useEffect(() => {
		if (products.length === 0) return;
		let current = true;
		api.purchases
			.productHistory(products)
			.then((found) => {
				if (!current) return;
				setHistory(
					new Map(found.map((h) => [productKey(h.store, h.product_id), h])),
				);
			})
			.catch((err) => console.warn("[purchase-history] could not read", err));
		return () => {
			current = false;
		};
	}, [key]);

	return history;
}
