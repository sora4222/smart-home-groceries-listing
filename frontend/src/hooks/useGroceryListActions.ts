import { useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import {
	api,
	type GroceryItem,
	type GroceryItemEdit,
	type ItemSelection,
	type NewGroceryItem,
	type OnDuplicate,
} from "#/lib/api";
import { restoreChoices } from "#/lib/item-restore";
import { ruleTermsDescription, termsAddedByRules } from "#/lib/rule-terms";

/**
 * Adding, editing, removing and committing on the grocery list page. Every
 * change's toast has Undo. Each goes through `lib/api` and then
 * `router.invalidate()`, so the loader stays the source of truth.
 *
 * `items` and `selections` are what the page shows now: what Undo restores.
 * `add` rejects with the backend's error so the page can ask about a duplicate.
 */
export function useGroceryListActions(
	items: GroceryItem[],
	selections: Map<string, ItemSelection>,
) {
	const router = useRouter();

	async function refresh() {
		try {
			await router.invalidate();
		} catch {
			// Saved; only the refresh failed (navigating away mid-refresh).
		}
	}

	/** Runs an undo, says how it went, and reloads the list. */
	async function undo(action: () => Promise<unknown>) {
		try {
			await action();
			toast.success("Undone");
		} catch {
			toast.error("Could not undo that — check the list.");
		}
		await refresh();
	}

	async function add(item: NewGroceryItem, onDuplicate: OnDuplicate) {
		const created = await api.grocery.add(item, onDuplicate);
		const before = items.find((i) => i.id === created.id);
		// A merge returns the existing item, whose chips no rule just added.
		const fromRules = before
			? []
			: termsAddedByRules(item.filter_terms, created.filter_terms);
		toast.success(`${created.name} is on the list`, {
			description: ruleTermsDescription(fromRules),
			action: {
				label: "Undo",
				onClick: () =>
					undo(() =>
						before
							? api.grocery.update(created.id, { quantity: before.quantity })
							: api.grocery.remove(created.id),
					),
			},
		});
		await refresh();
	}

	async function save(id: string, edit: GroceryItemEdit) {
		const before = items.find((i) => i.id === id);
		try {
			await api.grocery.update(id, edit);
		} catch {
			toast.error("Could not save that change — try again.");
			throw new Error("save failed");
		}
		if (before) {
			toast.success("Saved", {
				action: {
					label: "Undo",
					onClick: () =>
						undo(async () => {
							await api.grocery.update(id, {
								name: before.name,
								quantity: before.quantity,
								note: before.note ?? "",
								filter_terms: before.filter_terms,
							});
							// A new name or chips cleared the product; put it back.
							if (edit.name === undefined && edit.filter_terms === undefined)
								return;
							await restoreChoices(id, selections.get(id));
						}),
				},
			});
		}
		await refresh();
	}

	async function remove(id: string) {
		const before = items.find((i) => i.id === id);
		try {
			await api.grocery.remove(id);
		} catch {
			toast.error("Could not remove that item — try again.");
			return;
		}
		toast.success("Item removed", {
			action: before && {
				label: "Undo",
				onClick: () =>
					undo(async () => {
						const back = await api.grocery.restore({
							name: before.name,
							quantity: before.quantity,
							note: before.note ?? undefined,
							filter_terms: before.filter_terms,
						});
						await restoreChoices(back.id, selections.get(id));
					}),
			},
		});
		await refresh();
	}

	async function commit() {
		try {
			const locked = await api.grocery.commit();
			toast.success(
				`${locked.length} ${locked.length === 1 ? "item" : "items"} ready to order`,
				{
					action: {
						label: "Undo",
						onClick: () => undo(() => api.grocery.release(idsOf(locked))),
					},
				},
			);
		} catch {
			toast.error("Could not commit the list — try again.");
		}
		await refresh();
	}

	async function release() {
		try {
			const reopened = await api.grocery.release();
			toast.success("List reopened", {
				action: {
					label: "Undo",
					onClick: () => undo(() => api.grocery.commit(idsOf(reopened))),
				},
			});
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
