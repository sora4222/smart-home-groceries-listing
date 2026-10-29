/**
 * Putting a list item's products back after an undo.
 */
import { ApiError, api, type ItemChoices, type StoreId } from "#/lib/api";
import { type ChoiceUndo, productsAt } from "#/lib/choice-undo";

/**
 * Carries out `plan` against what the store holds now, changing only what
 * differs: products the change removed are chosen again first, then the
 * products it added are cleared, then the counts are set. A count-only undo
 * never asks the store, and a refused choice leaves the current products.
 *
 * Counts are set oldest first, leaving the newest: the backend takes any
 * change from the newer products, so the ones already set stay put.
 */
export async function applyUndo(itemId: string, plan: ChoiceUndo) {
	const now = (await api.selections.list()).find(
		(s) => s.grocery_item_id === itemId,
	);
	const current = productsAt(now ?? null, plan.store).map((p) => p.product_id);
	const wanted = plan.products.map((p) => p.product_id);
	const missing = wanted.filter((id) => !current.includes(id));
	const extra = current.filter((id) => !wanted.includes(id));

	for (const productId of missing) {
		await chooseMakingRoom(itemId, plan.store, productId, extra);
	}
	if (wanted.length === 0) {
		if (current.length > 0) await api.selections.clear(itemId, plan.store);
	} else {
		for (const productId of extra) {
			await api.selections.clear(itemId, plan.store, productId);
		}
	}

	// The backend's order now: kept products, then the ones chosen again.
	const order = [...current.filter((id) => wanted.includes(id)), ...missing];
	const counts = new Map(plan.products.map((p) => [p.product_id, p.quantity]));
	for (const productId of order.slice(0, -1)) {
		await api.selections.setQuantity(
			itemId,
			plan.store,
			productId,
			counts.get(productId) ?? 1,
		);
	}
	if (plan.buyAt) {
		await api.selections.buyAt([
			{ grocery_item_id: itemId, store: plan.buyAt },
		]);
	}
}

/**
 * Chooses a product again. When every slot is taken (409), the products
 * the undo would clear anyway make room, and it tries once more.
 */
async function chooseMakingRoom(
	itemId: string,
	store: StoreId,
	productId: string,
	extra: string[],
) {
	const choice = { store, product_id: productId };
	try {
		await api.selections.choose(itemId, choice);
	} catch (error) {
		if (!(error instanceof ApiError && error.status === 409) || !extra.length)
			throw error;
		for (const id of extra.splice(0)) {
			await api.selections.clear(itemId, store, id);
		}
		await api.selections.choose(itemId, choice);
	}
}

/**
 * Re-chooses every product `selection` had for a re-added `itemId`: the
 * other store first, then the store the order buys from.
 */
export async function restoreChoices(
	itemId: string,
	selection: ItemChoices | undefined,
): Promise<void> {
	if (!selection) return;
	const stores: StoreId[] = [
		...new Set(
			selection.also_chosen
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
