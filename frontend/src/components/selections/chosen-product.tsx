import { Button } from "#/components/ui/button";
import type { ItemSelection, StoreId } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * The products chosen for a list item, on the item's card: the one the order
 * buys, and the item's choice at the other store when there is one, each with
 * its size, store and shelf price when it was chosen. The order screen
 * re-prices them before anything is bought, and can switch the item between
 * its stores.
 *
 * With no choice it says so, so the household can see at a glance which items
 * still need a product. `onClear` (given the store to clear) is left out
 * where clearing is not allowed.
 */
export function ChosenProduct({
	itemName,
	selection,
	onClear,
}: {
	itemName: string;
	selection: ItemSelection | null;
	onClear?: (store: StoreId) => Promise<void>;
}) {
	if (!selection) {
		return (
			<p className="text-xs text-muted-foreground">No product chosen yet</p>
		);
	}

	const others = selection.also_chosen ?? [];
	return (
		<div className="flex flex-col gap-1">
			<ChoiceRow
				label={`Chosen product for ${itemName}`}
				clearLabel={`Clear the chosen product for ${itemName}`}
				selection={selection}
				onClear={onClear}
			/>
			{others.map((other) => (
				<ChoiceRow
					key={other.store}
					label={`Also chosen at ${other.store_name} for ${itemName}`}
					clearLabel={`Clear the ${other.store_name} product for ${itemName}`}
					note={`Also at ${other.store_name}`}
					selection={other}
					onClear={onClear}
				/>
			))}
		</div>
	);
}

/** One chosen product: what it is, and Clear. */
function ChoiceRow({
	label,
	clearLabel,
	note,
	selection,
	onClear,
}: {
	label: string;
	clearLabel: string;
	note?: string;
	selection: ItemSelection;
	onClear?: (store: StoreId) => Promise<void>;
}) {
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
			aria-label={label}
			className="flex items-center justify-between gap-2 rounded-md border border-border p-2"
		>
			<div className="flex min-w-0 flex-col">
				{note && <span className="text-xs text-muted-foreground">{note}</span>}
				<span className="text-sm font-medium">{title}</span>
				<span className="text-xs text-muted-foreground">{details}</span>
			</div>
			{onClear && (
				<Button
					size="sm"
					variant="ghost"
					onClick={() => onClear(selection.store)}
					aria-label={clearLabel}
				>
					Clear
				</Button>
			)}
		</section>
	);
}
