import { useState } from "react";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { AddItemForm } from "#/components/grocery/add-item-form";
import { DuplicatePrompt } from "#/components/grocery/duplicate-prompt";
import { GroceryItemCard } from "#/components/grocery/grocery-item-card";
import { GroceryList } from "#/components/grocery/grocery-list";
import {
	type DuplicateItemDetail,
	type NewGroceryItem,
	type OnDuplicate,
	api,
	duplicateDetail,
} from "#/lib/api";

export const Route = createFileRoute("/")({
	loader: () => api.grocery.list(),
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
 * Reading the list and adding to it are the two things this page exists for.
 * Mutations go through `lib/api` and then `router.invalidate()`, so the loader
 * stays the single source of truth and two open tabs cannot drift apart.
 */
function GroceryListPage() {
	const items = Route.useLoaderData();
	const router = useRouter();
	const [held, setHeld] = useState<HeldAddition | null>(null);

	async function addItem(
		item: NewGroceryItem,
		onDuplicate: OnDuplicate = "ask",
	) {
		try {
			const created = await api.grocery.add(item, onDuplicate);
			setHeld(null);
			toast.success(`${created.name} is on the list`);
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

			<GroceryList>
				{items.length === 0 ? (
					<GroceryList.Empty>
						Nothing on the list yet. Add an item above, or say "Hey Google, add
						milk to the shopping list" and accept it in Pending Requests.
					</GroceryList.Empty>
				) : (
					items.map((item) => (
						<GroceryList.Item key={item.id}>
							<GroceryItemCard item={item} />
						</GroceryList.Item>
					))
				)}
			</GroceryList>
		</div>
	);
}
