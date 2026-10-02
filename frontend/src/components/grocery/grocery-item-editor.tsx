import { useState } from "react";

import { TermListField } from "#/components/terms/term-list-field";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import type { GroceryItem, GroceryItemEdit } from "#/lib/api";
import { clampQuantity, MAX_QUANTITY, MIN_QUANTITY } from "#/lib/quantity";

interface GroceryItemEditorProps {
	item: GroceryItem;
	onSave: (edit: GroceryItemEdit) => Promise<void>;
	onCancel: () => void;
}

/**
 * The open editor for one list item: rename it, change how many, annotate it,
 * and add or drop a filter chip. Every draft is local state, so cancelling
 * leaves the item exactly as it was.
 */
export function GroceryItemEditor({
	item,
	onSave,
	onCancel,
}: GroceryItemEditorProps) {
	const [name, setName] = useState(item.name);
	const [quantity, setQuantity] = useState(String(item.quantity));
	const [note, setNote] = useState(item.note ?? "");
	const [terms, setTerms] = useState(item.filter_terms);
	const [busy, setBusy] = useState(false);

	async function save() {
		const trimmed = name.trim();
		if (!trimmed || busy) return;
		setBusy(true);
		try {
			await onSave({
				name: trimmed,
				quantity: clampQuantity(quantity),
				// An empty string is how the backend is told to clear the note.
				note: note.trim(),
				filter_terms: terms,
			});
		} catch {
			// The page reports the failure; the draft stays open for a retry.
		} finally {
			setBusy(false);
		}
	}

	return (
		<div className="flex flex-col gap-3 p-4">
			<div className="flex flex-col gap-2 sm:flex-row">
				<div className="flex-1">
					<label
						htmlFor={`name-${item.id}`}
						className="mb-1 block text-xs font-medium text-muted-foreground"
					>
						Item
					</label>
					<Input
						id={`name-${item.id}`}
						value={name}
						onChange={(event) => setName(event.target.value)}
					/>
				</div>
				<div className="w-full sm:w-24">
					<label
						htmlFor={`quantity-${item.id}`}
						className="mb-1 block text-xs font-medium text-muted-foreground"
					>
						Quantity
					</label>
					<Input
						id={`quantity-${item.id}`}
						type="number"
						min={MIN_QUANTITY}
						max={MAX_QUANTITY}
						value={quantity}
						onChange={(event) => setQuantity(event.target.value)}
						onBlur={() => setQuantity(String(clampQuantity(quantity)))}
					/>
				</div>
			</div>

			<div>
				<label
					htmlFor={`note-${item.id}`}
					className="mb-1 block text-xs font-medium text-muted-foreground"
				>
					Note
				</label>
				<Textarea
					id={`note-${item.id}`}
					value={note}
					maxLength={500}
					placeholder="Anything worth knowing before this is bought"
					onChange={(event) => setNote(event.target.value)}
				/>
			</div>

			<TermListField
				id={`filter-${item.id}`}
				label="Product filters"
				noun="filter"
				inputLabel="Add a product filter"
				addLabel="Add filter"
				placeholder="3 ply"
				maxLength={60}
				terms={terms}
				onChange={setTerms}
			/>

			<div className="flex gap-2">
				<Button
					type="button"
					size="sm"
					onClick={save}
					disabled={busy || name.trim().length === 0}
				>
					{busy ? "Saving…" : "Save"}
				</Button>
				<Button type="button" size="sm" variant="ghost" onClick={onCancel}>
					Cancel
				</Button>
			</div>
		</div>
	);
}
