import { GroceryItemCard } from "#/components/grocery/grocery-item-card";
import { GroceryList } from "#/components/grocery/grocery-list";
import type { GroceryItem, GroceryItemEdit } from "#/lib/api";

/**
 * One group of list items — the items under review, or the committed ones —
 * with the message to show when the group is empty. Renders nothing for an
 * empty group that has no message.
 */
export function GroceryItemSection({
	items,
	empty,
	onSave,
	onRemove,
}: {
	items: GroceryItem[];
	empty?: string;
	onSave: (id: string, edit: GroceryItemEdit) => Promise<void>;
	onRemove: (id: string) => Promise<void>;
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
					<GroceryItemCard item={item} onSave={onSave} onRemove={onRemove} />
				</GroceryList.Item>
			))}
		</GroceryList>
	);
}
