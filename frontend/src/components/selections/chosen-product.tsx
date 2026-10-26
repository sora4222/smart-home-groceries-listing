import type { ReactNode } from "react";

import { ProductImage } from "#/components/products/product-image";
import { CountSelect } from "#/components/selections/count-select";
import { Button } from "#/components/ui/button";
import type { ItemSelection, StoreId } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/** Clears one chosen product. */
type OnClear = (store: StoreId, productId: string) => Promise<void>;
/** Sets how many of one chosen product to buy. */
type OnSetCount = (
	store: StoreId,
	productId: string,
	count: number,
) => Promise<void>;

/**
 * The products chosen for a list item, on the item's card: the ones the
 * order buys, each with its count, and the item's choices at the other store.
 *
 * With two or more products to buy, each count is a dropdown; the counts
 * always add up to the item's quantity. With no choice it says so.
 * `onClear` and `onSetCount` are left out where changes are not allowed.
 */
export function ChosenProduct({
	itemName,
	selection,
	onClear,
	onSetCount,
}: {
	itemName: string;
	selection: ItemSelection | null;
	onClear?: OnClear;
	onSetCount?: OnSetCount;
}) {
	if (!selection) {
		return (
			<p className="text-xs text-muted-foreground">No product chosen yet</p>
		);
	}

	const bought = [selection, ...(selection.bought_with ?? [])];
	const total = bought.reduce((sum, s) => sum + s.quantity, 0);
	const several = bought.length > 1;
	const others = selection.also_chosen ?? [];
	return (
		<div className="flex flex-col gap-1">
			{bought.map((row, index) => (
				<ChoiceRow
					key={row.product_id}
					label={
						index === 0
							? `Chosen product for ${itemName}`
							: `Also buying ${productLabel(row)} for ${itemName}`
					}
					clearLabel={
						several
							? `Clear ${productLabel(row)} for ${itemName}`
							: `Clear the chosen product for ${itemName}`
					}
					selection={row}
					onClear={onClear}
					count={
						several && onSetCount ? (
							<CountSelect
								label={`How many ${productLabel(row)}`}
								value={row.quantity}
								max={total - (bought.length - 1)}
								onChange={(count) =>
									onSetCount(row.store, row.product_id, count)
								}
							/>
						) : (
							<span className="text-sm text-muted-foreground">
								×{row.quantity}
							</span>
						)
					}
				/>
			))}
			{others.map((other) => (
				<ChoiceRow
					key={`${other.store}-${other.product_id}`}
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

/** Brand, name and size: tells two sizes of one product apart. */
function productLabel(selection: ItemSelection): string {
	return [selection.brand, selection.name, selection.package_size]
		.filter(Boolean)
		.join(" ");
}

/** One chosen product: its picture, what it is, its count, and Clear. */
function ChoiceRow({
	label,
	clearLabel,
	note,
	selection,
	count,
	onClear,
}: {
	label: string;
	clearLabel: string;
	note?: string;
	selection: ItemSelection;
	count?: ReactNode;
	onClear?: OnClear;
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
			<div className="flex min-w-0 items-center gap-2">
				<ProductImage src={selection.image_url} name={title} size="sm" />
				<div className="flex min-w-0 flex-col">
					{note && (
						<span className="text-xs text-muted-foreground">{note}</span>
					)}
					<span className="text-sm font-medium">{title}</span>
					<span className="text-xs text-muted-foreground">{details}</span>
				</div>
			</div>
			<div className="flex shrink-0 items-center gap-1">
				{count}
				{onClear && (
					<Button
						size="sm"
						variant="ghost"
						onClick={() => onClear(selection.store, selection.product_id)}
						aria-label={clearLabel}
					>
						Clear
					</Button>
				)}
			</div>
		</section>
	);
}
