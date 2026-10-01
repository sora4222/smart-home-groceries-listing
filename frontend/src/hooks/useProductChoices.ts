import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { ApiError, api, type ProductChoice } from "#/lib/api";

/**
 * Choosing and clearing list items' products from the grocery list page.
 *
 * Each change goes through `lib/api` and then `router.invalidate()`, so the
 * page's loader stays the single source of truth. A refused choice (the store
 * no longer offers the product, or could not be searched) is explained with
 * the backend's own sentence, and the promise rejects so the button that
 * asked can recover.
 */
export function useProductChoices() {
	const router = useRouter();

	async function choose(itemId: string, choice: ProductChoice) {
		try {
			const saved = await api.selections.choose(itemId, choice);
			const title = [saved.brand, saved.name].filter(Boolean).join(" ");
			toast.success(`${title} chosen at ${saved.store_name}`);
			await router.invalidate();
		} catch (error) {
			toast.error(
				refusal(error) ?? "Could not choose that product — try again.",
			);
			throw error;
		}
	}

	async function clear(itemId: string) {
		try {
			await api.selections.clear(itemId);
			toast.success("Chosen product cleared");
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
