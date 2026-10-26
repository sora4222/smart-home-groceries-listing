import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	ApiError,
	api,
	type ItemSelection,
	type ProductChoice,
	type StoreId,
} from "#/lib/api";
import { type ChoiceUndo, undoAt } from "#/lib/choice-undo";
import { applyUndo } from "#/lib/item-restore";
import { toastWithUndo } from "#/lib/undo-toast";

/**
 * Choosing, counting and clearing list items' products.
 *
 * Each change goes through `lib/api` and then `router.invalidate()`, so the
 * page's loader stays the single source of truth. Every change's toast has
 * an Undo that puts the changed store back as it was (`lib/choice-undo`).
 * A refused choice (the store no longer offers the product, or could not be
 * searched) is explained with the backend's own sentence, and the promise
 * rejects so the button that asked can recover.
 *
 * `selections` is what the page shows now: each item's choices before the
 * change, which Undo restores.
 */
export function useProductChoices(selections: Map<string, ItemSelection>) {
	const router = useRouter();

	/** The success toast, whose Undo puts the store back by `plan`. */
	function done(message: string, itemId: string, plan: ChoiceUndo) {
		toastWithUndo(message, {
			undo: () => applyUndo(itemId, plan),
			failure: "Could not undo that. Check the item's products.",
			after: () => router.invalidate(),
		});
	}

	async function choose(itemId: string, choice: ProductChoice) {
		const plan = undoAt(selections.get(itemId) ?? null, choice.store);
		try {
			const saved = await api.selections.choose(itemId, choice);
			const title = [saved.brand, saved.name].filter(Boolean).join(" ");
			done(`${title} chosen at ${saved.store_name}`, itemId, plan);
			await router.invalidate();
		} catch (error) {
			toast.error(
				refusal(error) ?? "Could not choose that product — try again.",
			);
			throw error;
		}
	}

	async function clear(itemId: string, store: StoreId, productId?: string) {
		const plan = undoAt(selections.get(itemId) ?? null, store);
		try {
			await api.selections.clear(itemId, store, productId);
			done("Chosen product cleared", itemId, plan);
			await router.invalidate();
		} catch {
			toast.error("Could not clear the chosen product — try again.");
		}
	}

	async function setCount(
		itemId: string,
		store: StoreId,
		productId: string,
		count: number,
	) {
		const plan = undoAt(selections.get(itemId) ?? null, store);
		try {
			await api.selections.setQuantity(itemId, store, productId, count);
			done("Count changed", itemId, plan);
			await router.invalidate();
		} catch (error) {
			toast.error(refusal(error) ?? "Could not change the count — try again.");
		}
	}

	return { choose, clear, setCount };
}

/** The backend's explanation for a refused choice, when it gave one. */
function refusal(error: unknown): string | null {
	if (!(error instanceof ApiError)) return null;
	if (![409, 422, 503].includes(error.status)) return null;
	const detail = (error.body as { detail?: unknown } | undefined)?.detail;
	return typeof detail === "string" ? detail : null;
}
