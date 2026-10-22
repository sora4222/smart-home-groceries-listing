import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	ApiError,
	api,
	type ItemSelection,
	type ProductChoice,
	type StoreId,
} from "#/lib/api";
import { type ChoiceUndo, undoChoose, undoClear } from "#/lib/choice-undo";

/**
 * Choosing and clearing list items' products from the grocery list page.
 *
 * Each change goes through `lib/api` and then `router.invalidate()`, so the
 * page's loader stays the single source of truth. Every change's toast has
 * an Undo that puts the item's choices back as they were (`lib/choice-undo`).
 * A refused choice (the store no longer offers the product, or could not be
 * searched) is explained with the backend's own sentence, and the promise
 * rejects so the button that asked can recover.
 *
 * `selections` is what the page shows now: each item's choices before the
 * change, which Undo restores.
 */
export function useProductChoices(selections: Map<string, ItemSelection>) {
	const router = useRouter();

	async function undo(itemId: string, plan: ChoiceUndo) {
		try {
			if (plan.choose) await api.selections.choose(itemId, plan.choose);
			if (plan.clear) await api.selections.clear(itemId, plan.clear);
			if (plan.buyAt) {
				await api.selections.buyAt([
					{ grocery_item_id: itemId, store: plan.buyAt },
				]);
			}
			toast.success("Undone");
		} catch {
			toast.error("Could not undo that — check the item's products.");
		}
		await router.invalidate();
	}

	async function choose(itemId: string, choice: ProductChoice) {
		const plan = undoChoose(selections.get(itemId) ?? null, choice.store);
		try {
			const saved = await api.selections.choose(itemId, choice);
			const title = [saved.brand, saved.name].filter(Boolean).join(" ");
			toast.success(`${title} chosen at ${saved.store_name}`, {
				action: { label: "Undo", onClick: () => undo(itemId, plan) },
			});
			await router.invalidate();
		} catch (error) {
			toast.error(
				refusal(error) ?? "Could not choose that product — try again.",
			);
			throw error;
		}
	}

	async function clear(itemId: string, store: StoreId) {
		const plan = undoClear(selections.get(itemId) ?? null, store);
		try {
			await api.selections.clear(itemId, store);
			toast.success("Chosen product cleared", {
				action: { label: "Undo", onClick: () => undo(itemId, plan) },
			});
			await router.invalidate();
		} catch {
			toast.error("Could not clear the chosen product — try again.");
		}
	}

	return { choose, clear };
}

/** The backend's explanation for a refused choice, when it gave one. */
function refusal(error: unknown): string | null {
	if (!(error instanceof ApiError)) return null;
	if (![409, 422, 503].includes(error.status)) return null;
	const detail = (error.body as { detail?: unknown } | undefined)?.detail;
	return typeof detail === "string" ? detail : null;
}
