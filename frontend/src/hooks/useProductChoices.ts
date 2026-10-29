import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { ApiError, api, type ProductChoice, type StoreId } from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

/**
 * Choosing, counting and clearing list items' products.
 *
 * Each change goes through `lib/api` and then `router.invalidate()`, so the
 * page's loader stays the single source of truth. Before each change the
 * item's products are saved as a snapshot, and the toast's Undo puts that
 * snapshot back in one step. A refused choice (the store no longer offers
 * the product, or could not be searched) is explained with the backend's
 * own sentence, and the promise rejects so the button that asked can
 * recover.
 */
export function useProductChoices() {
	const router = useRouter();

	/**
	 * Saves the item's products, runs `change`, and shows the toast `message`
	 * names, whose Undo puts the products back.
	 */
	async function withUndo<T>(
		itemId: string,
		change: () => Promise<T>,
		message: (result: T) => string,
	) {
		const snapshot = await api.selections.snapshot(itemId);
		const result = await change();
		toastWithUndo(message(result), {
			undo: () => api.selections.restore(snapshot.id),
			failure: "Could not undo that. Check the item's products.",
			after: () => router.invalidate(),
		});
		await router.invalidate();
	}

	async function choose(itemId: string, choice: ProductChoice) {
		try {
			await withUndo(
				itemId,
				() => api.selections.choose(itemId, choice),
				(saved) =>
					`${[saved.brand, saved.name].filter(Boolean).join(" ")} chosen at ${saved.store_name}`,
			);
		} catch (error) {
			toast.error(
				refusal(error) ?? "Could not choose that product — try again.",
			);
			throw error;
		}
	}

	async function clear(itemId: string, store: StoreId, productId?: string) {
		try {
			await withUndo(
				itemId,
				() => api.selections.clear(itemId, store, productId),
				() => "Chosen product cleared",
			);
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
		try {
			await withUndo(
				itemId,
				() => api.selections.setQuantity(itemId, store, productId, count),
				() => "Count changed",
			);
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
