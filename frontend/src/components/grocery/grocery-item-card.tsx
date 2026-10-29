import { type ReactNode, useState } from "react";

import { GroceryItemEditor } from "#/components/grocery/grocery-item-editor";
import { ChosenProduct } from "#/components/selections/chosen-product";
import { TermChips } from "#/components/terms/term-chips";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import {
	Card,
	CardAction,
	CardContent,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import type {
	GroceryItem,
	GroceryItemEdit,
	ItemChoices,
	StoreId,
} from "#/lib/api";

interface GroceryItemCardProps {
	item: GroceryItem;
	onSave: (id: string, edit: GroceryItemEdit) => Promise<void>;
	onRemove: (id: string) => Promise<void>;
	/** The product chosen for this item, if any. */
	selection?: ItemChoices | null;
	/** The "Select item" link to the item's product page. */
	selectLink?: ReactNode;
	/** Forgets the item's chosen product; leave out to hide "Clear". */
	onClearChoice?: (
		id: string,
		store: StoreId,
		productId: string,
	) => Promise<void>;
	/** Sets how many of one of the item's products to buy. */
	onSetCount?: (
		id: string,
		store: StoreId,
		productId: string,
		count: number,
	) => Promise<void>;
}

/**
 * One row of the grocery list, in one of two modes.
 *
 * Reading: the name, how many, where it came from, the note somebody left and
 * the filter chips — which can be taken off from here, without editing, as the
 * spec's Item Rules section requires.
 *
 * Editing: the whole item at once, via [`GroceryItemEditor`].
 *
 * A committed item is shown but not editable: the household has locked the list
 * in for purchase and has to release it to change anything. Its product can
 * still be selected, since the order screen needs one.
 */
export function GroceryItemCard({
	item,
	onSave,
	onRemove,
	selection = null,
	selectLink,
	onClearChoice,
	onSetCount,
}: GroceryItemCardProps) {
	const [editing, setEditing] = useState(false);
	const [confirmingRemoval, setConfirmingRemoval] = useState(false);
	const locked = item.status === "committed";

	async function save(edit: GroceryItemEdit) {
		await onSave(item.id, edit);
		setEditing(false);
	}

	if (editing && !locked) {
		return (
			<Card data-testid="grocery-item" data-item-name={item.name}>
				<GroceryItemEditor
					item={item}
					onSave={save}
					onCancel={() => setEditing(false)}
				/>
			</Card>
		);
	}

	return (
		<Card
			data-testid="grocery-item"
			data-item-name={item.name}
			className="gap-2 py-4"
		>
			<CardHeader className="px-4">
				<CardTitle className="font-medium">{item.name}</CardTitle>
				<CardAction className="flex items-center gap-2">
					{locked && <Badge variant="outline">Committed</Badge>}
					<Badge variant="secondary" aria-label={`Quantity ${item.quantity}`}>
						×{item.quantity}
					</Badge>
				</CardAction>
			</CardHeader>

			<CardContent className="flex flex-col gap-2 px-4">
				{item.note && <p className="text-sm">{item.note}</p>}

				<TermChips
					terms={item.filter_terms}
					onRemove={
						locked
							? undefined
							: (term) =>
									onSave(item.id, {
										// The whole remaining list is sent; that is how the
										// backend is told one chip has gone.
										filter_terms: item.filter_terms.filter((t) => t !== term),
									})
					}
				/>

				<ChosenProduct
					itemName={item.name}
					selection={selection}
					onClear={
						onClearChoice &&
						((store, productId) => onClearChoice(item.id, store, productId))
					}
					onSetCount={
						onSetCount &&
						((store, productId, count) =>
							onSetCount(item.id, store, productId, count))
					}
				/>

				<div className="flex flex-wrap items-center justify-between gap-2">
					<span className="text-xs text-muted-foreground">
						Added via {item.source === "voice" ? "voice" : "the web app"}
					</span>

					<div className="flex flex-wrap gap-2">
						{selectLink}
						{!locked &&
							(confirmingRemoval ? (
								<>
									<Button
										size="sm"
										variant="destructive"
										onClick={() => onRemove(item.id)}
										aria-label={`Confirm removing ${item.name}`}
									>
										Remove
									</Button>
									<Button
										size="sm"
										variant="ghost"
										onClick={() => setConfirmingRemoval(false)}
									>
										Keep
									</Button>
								</>
							) : (
								<>
									<Button
										size="sm"
										variant="outline"
										onClick={() => setEditing(true)}
										aria-label={`Edit ${item.name}`}
									>
										Edit
									</Button>
									<Button
										size="sm"
										variant="destructive"
										onClick={() => setConfirmingRemoval(true)}
										aria-label={`Remove ${item.name}`}
									>
										Remove
									</Button>
								</>
							))}
					</div>
				</div>
			</CardContent>
		</Card>
	);
}
