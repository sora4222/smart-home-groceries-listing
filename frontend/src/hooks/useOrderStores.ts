import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	api,
	type ItemSelection,
	type OrderOption,
	type OrderStorePick,
} from "#/lib/api";

/**
 * Using one of the order's options: each item is bought at that option's
 * store. The toast's Undo puts every moved item back at the store it was
 * bought from before (`selections` is what the page shows now).
 */
export function useOrderStores(selections: Map<string, ItemSelection>) {
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
		toast.success(`The order now uses “${option.label}”`, {
			action:
				previous.length > 0
					? { label: "Undo", onClick: () => undo(previous) }
					: undefined,
		});
		await router.invalidate();
	}

	async function undo(previous: OrderStorePick[]) {
		try {
			await api.selections.buyAt(previous);
			toast.success("Undone");
		} catch {
			toast.error("Could not undo — check the order.");
		}
		await router.invalidate();
	}

	return { use };
}
