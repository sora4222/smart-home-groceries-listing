/**
 * Putting a list item's products back after an undo: the other store's
 * choice first, then the one the order buys, so that one ends up bought.
 */
import { api, type ItemSelection } from "#/lib/api";

/** Re-chooses `selection` (and its other-store choices) for `itemId`. */
export async function restoreChoices(
	itemId: string,
	selection: ItemSelection | undefined,
): Promise<void> {
	if (!selection) return;
	for (const choice of [...(selection.also_chosen ?? []), selection]) {
		await api.selections.choose(itemId, {
			store: choice.store,
			product_id: choice.product_id,
		});
	}
}
