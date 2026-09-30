import { useState } from "react";

import { FilterChips } from "#/components/grocery/filter-chips";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import type { GroceryItem, GroceryItemEdit } from "#/lib/api";
import { MAX_QUANTITY, MIN_QUANTITY, clampQuantity } from "#/lib/quantity";

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
	const [draftTerm, setDraftTerm] = useState("");
	const [busy, setBusy] = useState(false);

	function addTerm() {
		const term = draftTerm.trim();
		const alreadyThere = terms.some(
			(existing) => existing.toLowerCase() === term.toLowerCase(),
		);
		if (!term || alreadyThere || terms.length >= 10) {
			setDraftTerm("");
			return;
		}
		setTerms([...terms, term]);
		setDraftTerm("");
	}

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

			<div className="flex flex-col gap-2">
				<span className="text-xs font-medium text-muted-foreground">
					Product filters
				</span>
				<FilterChips
					terms={terms}
					onRemove={(term) => setTerms(terms.filter((t) => t !== term))}
				/>
				<div className="flex gap-2">
					<label className="sr-only" htmlFor={`filter-${item.id}`}>
						Add a product filter
					</label>
					<Input
						id={`filter-${item.id}`}
						value={draftTerm}
						maxLength={60}
						placeholder="3 ply"
						onChange={(event) => setDraftTerm(event.target.value)}
						onKeyDown={(event) => {
							if (event.key === "Enter") {
								// Inside a form this would submit the page instead.
								event.preventDefault();
								addTerm();
							}
						}}
					/>
					<Button
						type="button"
						variant="outline"
						onClick={addTerm}
						disabled={draftTerm.trim().length === 0 || terms.length >= 10}
					>
						Add filter
					</Button>
				</div>
			</div>

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
