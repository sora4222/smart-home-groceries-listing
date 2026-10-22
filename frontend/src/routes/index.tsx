import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

import { AddItemForm } from "#/components/grocery/add-item-form";
import { CommitBar } from "#/components/grocery/commit-bar";
import { DuplicatePrompt } from "#/components/grocery/duplicate-prompt";
import { GroceryItemSection } from "#/components/grocery/grocery-item-section";
import { SendToStore } from "#/components/trolley/send-to-store";
import { useProductChoices } from "#/hooks/useProductChoices";
import {
	api,
	type DuplicateItemDetail,
	duplicateDetail,
	type GroceryItemEdit,
	type NewGroceryItem,
	type OnDuplicate,
	selectionsByItem,
} from "#/lib/api";
import { chosenAtStore } from "#/lib/chosen-at-store";
import { ruleTermsDescription, termsAddedByRules } from "#/lib/rule-terms";

export const Route = createFileRoute("/")({
	// The list and every item's chosen product, read together so a card never
	// shows an item without knowing its choice.
	loader: async () => {
		const [items, selections] = await Promise.all([
			api.grocery.list(),
			api.selections.list(),
		]);
		return { items, selections: selectionsByItem(selections) };
	},
	component: GroceryListPage,
});

/** An addition the backend asked a question about, held until the user answers. */
interface HeldAddition {
	item: NewGroceryItem;
	detail: DuplicateItemDetail;
}

/**
 * The grocery list — the application's main page.
 *
 * Reading the list and adding to it are the two things this page exists for;
 * reviewing, annotating, choosing each item's product and finally committing
 * the list for purchase are what the rest of it is about.
 *
 * Mutations go through `lib/api` and then `router.invalidate()`, so the loader
 * stays the single source of truth and two open tabs cannot drift apart.
 */
function GroceryListPage() {
	const { items, selections } = Route.useLoaderData();
	const router = useRouter();
	const choices = useProductChoices(selections);
	const [held, setHeld] = useState<HeldAddition | null>(null);

	const underReview = items.filter((item) => item.status === "active");
	const committed = items.filter((item) => item.status === "committed");
	const chosenAtWoolworths = chosenAtStore(items, selections, "woolworths");
	const chosenAtColes = chosenAtStore(items, selections, "coles");

	async function addItem(
		item: NewGroceryItem,
		onDuplicate: OnDuplicate = "ask",
	) {
		try {
			const created = await api.grocery.add(item, onDuplicate);
			setHeld(null);
			// A merge returns the existing item, whose chips no rule just added.
			const fromRules =
				onDuplicate === "merge"
					? []
					: termsAddedByRules(item.filter_terms, created.filter_terms);
			toast.success(`${created.name} is on the list`, {
				description: ruleTermsDescription(fromRules),
			});
			await router.invalidate();
		} catch (error) {
			const detail = duplicateDetail(error);
			if (detail) {
				// The name is already on the list; let the household decide.
				setHeld({ item, detail });
				return;
			}
			toast.error("Could not add that item — try again.");
			throw error;
		}
	}

	async function saveItem(id: string, edit: GroceryItemEdit) {
		try {
			await api.grocery.update(id, edit);
			await router.invalidate();
		} catch {
			toast.error("Could not save that change — try again.");
			throw new Error("save failed");
		}
	}

	async function removeItem(id: string) {
		try {
			await api.grocery.remove(id);
			toast.success("Item removed from the list");
			await router.invalidate();
		} catch {
			toast.error("Could not remove that item — try again.");
		}
	}

	async function commitList() {
		try {
			const locked = await api.grocery.commit();
			toast.success(
				`${locked.length} ${locked.length === 1 ? "item" : "items"} locked in for purchase`,
			);
			await router.invalidate();
		} catch {
			toast.error("Could not commit the list — try again.");
		}
	}

	async function releaseList() {
		try {
			await api.grocery.release();
			toast.success("List reopened for editing");
			await router.invalidate();
		} catch {
			toast.error("Could not release the list — try again.");
		}
	}

	return (
		<div className="flex flex-col gap-4">
			<h1 className="text-lg font-semibold">Grocery List</h1>

			<AddItemForm onAdd={(item) => addItem(item)} />

			{held && (
				<DuplicatePrompt
					detail={held.detail}
					onMerge={() => addItem(held.item, "merge")}
					onSeparate={() => addItem(held.item, "separate")}
					onCancel={() => setHeld(null)}
				/>
			)}

			<GroceryItemSection
				items={underReview}
				selections={selections}
				onSave={saveItem}
				onRemove={removeItem}
				onChoose={choices.choose}
				onClearChoice={choices.clear}
				empty={
					committed.length > 0
						? "Nothing new since the list was committed."
						: 'Nothing on the list yet. Add an item above, or say "Hey Google, add milk to the shopping list" and accept it in Pending Requests.'
				}
			/>

			<CommitBar
				activeCount={underReview.length}
				committedCount={committed.length}
				onCommit={commitList}
				onRelease={releaseList}
			/>

			<div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
				<SendToStore store="woolworths" chosenCount={chosenAtWoolworths}>
					<SendToStore.Trigger />
					<SendToStore.Content />
				</SendToStore>
				<SendToStore store="coles" chosenCount={chosenAtColes}>
					<SendToStore.Trigger />
					<SendToStore.Content />
				</SendToStore>
			</div>

			{committed.length > 0 && (
				<>
					<h2 className="text-sm font-semibold text-muted-foreground">
						Committed for purchase
					</h2>
					<GroceryItemSection
						items={committed}
						selections={selections}
						onSave={saveItem}
						onRemove={removeItem}
						onChoose={choices.choose}
						onClearChoice={choices.clear}
					/>
				</>
			)}
		</div>
	);
}
