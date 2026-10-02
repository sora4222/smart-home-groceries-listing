import { useState } from "react";

import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import type { NewGroceryItem } from "#/lib/api";
import { clampQuantity, MAX_QUANTITY, MIN_QUANTITY } from "#/lib/quantity";

interface AddItemFormProps {
	/** Resolves once the item is on the list, or rejects to keep the draft. */
	onAdd: (item: NewGroceryItem) => Promise<void>;
}

/**
 * The add-item form at the top of the grocery list — the one thing a household
 * member does most often, so it is a single row: name, quantity, Add.
 *
 * Annotating an item (note, filter chips) happens on the item itself once it is
 * on the list, which keeps this path to two keystrokes and a return.
 */
export function AddItemForm({ onAdd }: AddItemFormProps) {
	const [name, setName] = useState("");
	// Held as typed rather than as a number: coercing on every keystroke would
	// snap an emptied field back to 1, so the next digit would append to it.
	const [quantity, setQuantity] = useState("1");
	const [busy, setBusy] = useState(false);

	const trimmed = name.trim();

	async function submit(event: React.FormEvent) {
		event.preventDefault();
		if (!trimmed || busy) return;
		setBusy(true);
		try {
			await onAdd({ name: trimmed, quantity: clampQuantity(quantity) });
			setName("");
			setQuantity("1");
		} catch {
			// The page reports the failure; the form's job is to keep the draft
			// so nothing has to be retyped.
		} finally {
			setBusy(false);
		}
	}

	return (
		<form
			onSubmit={submit}
			aria-label="Add a grocery item"
			className="flex flex-col gap-2 sm:flex-row sm:items-end"
		>
			<div className="flex-1">
				<label
					htmlFor="new-item-name"
					className="mb-1 block text-xs font-medium text-muted-foreground"
				>
					Item
				</label>
				<Input
					id="new-item-name"
					value={name}
					placeholder="Milk"
					autoComplete="off"
					onChange={(event) => setName(event.target.value)}
				/>
			</div>
			<div className="w-full sm:w-24">
				<label
					htmlFor="new-item-quantity"
					className="mb-1 block text-xs font-medium text-muted-foreground"
				>
					Quantity
				</label>
				<Input
					id="new-item-quantity"
					type="number"
					min={MIN_QUANTITY}
					max={MAX_QUANTITY}
					value={quantity}
					onChange={(event) => setQuantity(event.target.value)}
					onBlur={() => setQuantity(String(clampQuantity(quantity)))}
				/>
			</div>
			<Button
				type="submit"
				className="w-full sm:w-auto"
				disabled={busy || trimmed.length === 0}
			>
				{busy ? "Adding…" : "Add item"}
			</Button>
		</form>
	);
}
