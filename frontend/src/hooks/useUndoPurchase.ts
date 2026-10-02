/**
 * Undo for a shop saved as bought by mistake.
 *
 * `undo(order)` deletes the saved shop; the backend puts its items back on
 * the list. A toast says what happened. `busyId` is the shop being undone,
 * so its button can show it is working.
 */
import { useCallback, useState } from "react";
import { toast } from "sonner";

import { ApiError, api, type PurchaseOrder } from "#/lib/api";

export interface UndoPurchase {
	busyId: string | null;
	/** Resolves `true` when the shop is no longer saved (undone, or already gone). */
	undo: (order: PurchaseOrder) => Promise<boolean>;
}

export function useUndoPurchase(): UndoPurchase {
	const [busyId, setBusyId] = useState<string | null>(null);

	const undo = useCallback(async (order: PurchaseOrder) => {
		setBusyId(order.id);
		try {
			const { items_restored } = await api.purchases.undo(order.id);
			toast(undoneMessage(order.store_name, items_restored));
			return true;
		} catch (err) {
			if (err instanceof ApiError && err.status === 404) {
				toast("That shop was already undone.");
				return true;
			}
			console.error("[purchases] could not undo", err);
			toast("Could not undo just now — try again.");
			return false;
		} finally {
			setBusyId(null);
		}
	}, []);

	return { busyId, undo };
}

/** What Undo did, in one sentence. */
export function undoneMessage(
	storeName: string,
	itemsRestored: number,
): string {
	const back =
		itemsRestored === 0
			? "No items needed to go back on the list."
			: itemsRestored === 1
				? "1 item is back on your list."
				: `${itemsRestored} items are back on your list.`;
	return `Undone: the ${storeName} shop is no longer saved as bought. ${back}`;
}
