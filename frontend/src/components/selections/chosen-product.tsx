import { Button } from "#/components/ui/button";
import type { ItemSelection } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * The product chosen for a list item, on the item's card: what it is, its
 * size, the store, and its shelf price when it was chosen. The order screen
 * re-prices it before anything is bought.
 *
 * With no choice it says so, so the household can see at a glance which items
 * still need a product. `onClear` is left out where clearing is not allowed.
 */
export function ChosenProduct({
	itemName,
	selection,
	onClear,
}: {
	itemName: string;
	selection: ItemSelection | null;
	onClear?: () => Promise<void>;
}) {
	if (!selection) {
		return (
			<p className="text-xs text-muted-foreground">No product chosen yet</p>
		);
	}

	const title = [selection.brand, selection.name].filter(Boolean).join(" ");
	const details = [
		selection.package_size,
		selection.store_name,
		selection.price ? `${formatMoney(selection.price)} each` : null,
	]
		.filter(Boolean)
		.join(" · ");

	return (
		<section
			aria-label={`Chosen product for ${itemName}`}
			className="flex items-center justify-between gap-2 rounded-md border border-border p-2"
		>
			<div className="flex min-w-0 flex-col">
				<span className="text-sm font-medium">{title}</span>
				<span className="text-xs text-muted-foreground">{details}</span>
			</div>
			{onClear && (
				<Button
					size="sm"
					variant="ghost"
					onClick={() => onClear()}
					aria-label={`Clear the chosen product for ${itemName}`}
				>
					Clear
				</Button>
			)}
		</section>
	);
}
