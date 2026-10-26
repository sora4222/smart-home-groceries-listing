/**
 * Putting a list item's products back after an undo.
 */
import { api, type ItemSelection, type StoreId } from "#/lib/api";
import { type ChoiceUndo, productsAt } from "#/lib/choice-undo";

/**
 * Carries out `plan`: clears the store, chooses its products again oldest
 * first, sets their counts, then buys at the earlier store.
 *
 * Counts are set oldest first, leaving the newest: the backend takes any
 * change from the newer products, so the ones already set stay put.
 */
export async function applyUndo(itemId: string, plan: ChoiceUndo) {
	await api.selections.clear(itemId, plan.store);
	for (const product of plan.products) {
		await api.selections.choose(itemId, {
			store: plan.store,
			product_id: product.product_id,
		});
	}
	if (plan.products.length > 1) {
		for (const product of plan.products.slice(0, -1)) {
			await api.selections.setQuantity(
				itemId,
				plan.store,
				product.product_id,
				product.quantity,
			);
		}
	}
	if (plan.buyAt) {
		await api.selections.buyAt([
			{ grocery_item_id: itemId, store: plan.buyAt },
		]);
	}
}

/**
 * Re-chooses every product `selection` had for a re-added `itemId`: the
 * other store first, then the store the order buys from.
 */
export async function restoreChoices(
	itemId: string,
	selection: ItemSelection | undefined,
): Promise<void> {
	if (!selection) return;
	const stores: StoreId[] = [
		...new Set(
			(selection.also_chosen ?? [])
				.map((s) => s.store)
				.filter((store) => store !== selection.store),
		),
		selection.store,
	];
	for (const store of stores) {
		await applyUndo(itemId, {
			store,
			products: productsAt(selection, store),
			buyAt: null,
		});
	}
}
