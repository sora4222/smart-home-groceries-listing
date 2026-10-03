import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import {
	api,
	type DislikedProduct,
	type DislikeOverride,
	type ProductChoice,
	type ProductDislike,
} from "#/lib/api";

/** Every dislike and override, as one price comparison sees them. */
export interface ItemDislikes {
	dislikes: ProductDislike[];
	overrides: DislikeOverride[];
}

const empty: ItemDislikes = { dislikes: [], overrides: [] };

/**
 * The household's dislikes and the overrides for one item's price
 * comparison, with the actions that change them.
 *
 * Loads once when mounted (the comparison is open) and again after every
 * change, so what the sheet shows is what the backend saved. If loading
 * fails the sheet still works; it just shows no warnings.
 */
export function useItemDislikes(itemId: string) {
	const [data, setData] = useState<ItemDislikes>(empty);
	const [version, setVersion] = useState(0);

	// biome-ignore lint/correctness/useExhaustiveDependencies: version is a deliberate reload trigger.
	useEffect(() => {
		let current = true;
		Promise.all([api.dislikes.list(), api.dislikes.overrides()])
			.then(([dislikes, overrides]) => {
				if (current) setData({ dislikes, overrides });
			})
			.catch(() => {
				// No warnings is better than no comparison.
			});
		return () => {
			current = false;
		};
	}, [itemId, version]);

	const reload = useCallback(() => setVersion((n) => n + 1), []);

	async function run(
		action: () => Promise<unknown>,
		done: string,
		failed: string,
	) {
		try {
			await action();
			toast.success(done);
			reload();
		} catch {
			toast.error(failed);
		}
	}

	return {
		...data,
		dislike: (product: DislikedProduct) =>
			run(
				() => api.dislikes.dislike(product),
				`${product.name} marked as disliked`,
				"Could not save your dislike — try again.",
			),
		removeMine: (choice: ProductChoice) =>
			run(
				() => api.dislikes.removeMine(choice),
				"Your dislike is removed",
				"Could not remove your dislike — try again.",
			),
		override: (choice: ProductChoice) =>
			run(
				() => api.dislikes.override(itemId, choice),
				"OK to buy it this time",
				"Could not save that — try again.",
			),
		clearOverride: (choice: ProductChoice) =>
			run(
				() => api.dislikes.clearOverride(itemId, choice),
				"The dislike counts again for this order",
				"Could not undo that — try again.",
			),
	};
}
