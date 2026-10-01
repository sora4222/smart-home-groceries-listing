import { GroceryItemCard } from "#/components/grocery/grocery-item-card";
import { GroceryList } from "#/components/grocery/grocery-list";
import type {
	GroceryItem,
	GroceryItemEdit,
	ItemSelection,
	ProductChoice,
} from "#/lib/api";

/**
 * One group of list items — the items under review, or the committed ones —
 * with the message to show when the group is empty, and each item's chosen
 * product from `selections`. Renders nothing for an empty group that has no
 * message.
 */
export function GroceryItemSection({
	items,
	empty,
	selections,
	onSave,
	onRemove,
	onChoose,
	onClearChoice,
}: {
	items: GroceryItem[];
	empty?: string;
	selections: Map<string, ItemSelection>;
	onSave: (id: string, edit: GroceryItemEdit) => Promise<void>;
	onRemove: (id: string) => Promise<void>;
	onChoose: (id: string, choice: ProductChoice) => Promise<void>;
	onClearChoice: (id: string) => Promise<void>;
}) {
	if (items.length === 0) {
		return empty ? (
			<GroceryList>
				<GroceryList.Empty>{empty}</GroceryList.Empty>
			</GroceryList>
		) : null;
	}

	return (
		<GroceryList>
			{items.map((item) => (
				<GroceryList.Item key={item.id}>
					<GroceryItemCard
						item={item}
						onSave={onSave}
						onRemove={onRemove}
						selection={selections.get(item.id) ?? null}
						onChoose={onChoose}
						onClearChoice={onClearChoice}
					/>
				</GroceryList.Item>
			))}
		</GroceryList>
	);
}
