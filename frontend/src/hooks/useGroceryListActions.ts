import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	api,
	type GroceryItem,
	type GroceryItemEdit,
	type NewGroceryItem,
	type OnDuplicate,
} from "#/lib/api";
import { ruleTermsDescription, termsAddedByRules } from "#/lib/rule-terms";
import { toastWithUndo } from "#/lib/undo-toast";

/**
 * Adding, editing, removing and committing on the grocery list page. Every
 * change's toast has Undo. Each goes through `lib/api` and then
 * `router.invalidate()`, so the loader stays the source of truth.
 *
 * `items` is what the page shows now: what Undo restores. An edit or a
 * removal first saves the item's products as a snapshot, which Undo puts
 * back (a new name or chips, or removing the item, forgets them).
 * `add` rejects with the backend's error so the page can ask about a duplicate.
 */
export function useGroceryListActions(items: GroceryItem[]) {
	const router = useRouter();

	async function refresh() {
		try {
			await router.invalidate();
		} catch {
			// Saved; only the refresh failed (navigating away mid-refresh).
		}
	}

	/** The success toast, whose Undo runs `undo` and reloads the list. */
	function done(
		message: string,
		undo: () => Promise<unknown>,
		description?: string,
	) {
		toastWithUndo(message, {
			undo,
			description,
			failure: "Could not undo that. Check the list.",
			after: refresh,
		});
	}

	async function add(item: NewGroceryItem, onDuplicate: OnDuplicate) {
		const created = await api.grocery.add(item, onDuplicate);
		const before = items.find((i) => i.id === created.id);
		// A merge returns the existing item, whose chips no rule just added.
		const fromRules = before
			? []
			: termsAddedByRules(item.filter_terms, created.filter_terms);
		done(
			`${created.name} is on the list`,
			() =>
				before
					? api.grocery.update(created.id, { quantity: before.quantity })
					: api.grocery.remove(created.id),
			ruleTermsDescription(fromRules),
		);
		await refresh();
	}

	async function save(id: string, edit: GroceryItemEdit) {
		const before = items.find((i) => i.id === id);
		try {
			const snapshot = await api.selections.snapshot(id);
			await api.grocery.update(id, edit);
			if (before) {
				done("Saved", async () => {
					await api.grocery.update(id, {
						name: before.name,
						quantity: before.quantity,
						note: before.note ?? "",
						filter_terms: before.filter_terms,
					});
					await api.selections.restore(snapshot.id);
				});
			}
		} catch {
			toast.error("Could not save that change — try again.");
			throw new Error("save failed");
		}
		await refresh();
	}

	async function remove(id: string) {
		const before = items.find((i) => i.id === id);
		try {
			const snapshot = await api.selections.snapshot(id);
			await api.grocery.remove(id);
			if (before) {
				done("Item removed", async () => {
					const back = await api.grocery.restore({
						name: before.name,
						quantity: before.quantity,
						note: before.note ?? undefined,
						filter_terms: before.filter_terms,
					});
					await api.selections.restore(snapshot.id, back.id);
				});
			} else {
				toast.success("Item removed");
			}
		} catch {
			toast.error("Could not remove that item — try again.");
			return;
		}
		await refresh();
	}

	async function commit() {
		try {
			const locked = await api.grocery.commit();
			done(
				`${locked.length} ${locked.length === 1 ? "item" : "items"} ready to order`,
				() => api.grocery.release(idsOf(locked)),
			);
		} catch {
			toast.error("Could not commit the list — try again.");
		}
		await refresh();
	}

	async function release() {
		try {
			const reopened = await api.grocery.release();
			done("List reopened", () => api.grocery.commit(idsOf(reopened)));
		} catch {
			toast.error("Could not reopen the list — try again.");
		}
		await refresh();
	}

	return { add, save, remove, commit, release };
}

/** Undo moves back only the items a commit or release moved. */
function idsOf(items: GroceryItem[]): string[] {
	return items.map((item) => item.id);
}
