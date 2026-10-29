import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	api,
	type ItemChoices,
	type OrderOption,
	type OrderStorePick,
} from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

/**
 * Using one of the order's options: each item is bought at that option's
 * store. The toast's Undo puts every moved item back at the store it was
 * bought from before (`selections` is what the page shows now).
 */
export function useOrderStores(selections: Map<string, ItemChoices>) {
	const router = useRouter();

	async function use(option: OrderOption) {
		const previous: OrderStorePick[] = option.picks.flatMap((pick) => {
			const before = selections.get(pick.grocery_item_id);
			return before && before.store !== pick.store
				? [{ grocery_item_id: pick.grocery_item_id, store: before.store }]
				: [];
		});
		try {
			await api.selections.buyAt(option.picks);
		} catch {
			toast.error("Could not switch stores — check prices again.");
			return;
		}
		const message = `The order now uses "${option.label}"`;
		if (previous.length > 0) {
			toastWithUndo(message, {
				undo: () => api.selections.buyAt(previous),
				failure: "Could not undo. Check the order.",
				after: () => router.invalidate(),
			});
		} else {
			toast.success(message);
		}
		await router.invalidate();
	}

	return { use };
}
