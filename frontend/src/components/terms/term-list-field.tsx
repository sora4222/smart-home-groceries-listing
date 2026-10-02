import { useState } from "react";

import { TermChips } from "#/components/terms/term-chips";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { addTerm, MAX_TERMS } from "#/lib/terms";

interface TermListFieldProps {
	/** Unique on the page; the text input's id. */
	id: string;
	/** The visible heading over the chips, e.g. "Product filters". */
	label: string;
	/** What one chip is, for its remove button: "Remove filter 3 ply". */
	noun: string;
	/** The text input's accessible name, e.g. "Add a product filter". */
	inputLabel: string;
	/** The add button's text, e.g. "Add filter". */
	addLabel: string;
	placeholder?: string;
	/** Longest single term the backend accepts. */
	maxLength: number;
	terms: readonly string[];
	onChange: (terms: string[]) => void;
}

/**
 * An editable chip list: the chips so far, each removable, and a text input
 * with an Add button. Enter adds too, without submitting a surrounding form.
 *
 * The chips belong to the caller; only the half-typed term is local, so
 * cancelling the surrounding editor throws nothing away that was saved.
 */
export function TermListField({
	id,
	label,
	noun,
	inputLabel,
	addLabel,
	placeholder,
	maxLength,
	terms,
	onChange,
}: TermListFieldProps) {
	const [draft, setDraft] = useState("");
	const full = terms.length >= MAX_TERMS;

	function add() {
		onChange(addTerm(terms, draft));
		setDraft("");
	}

	return (
		<div className="flex flex-col gap-2">
			<span className="text-xs font-medium text-muted-foreground">{label}</span>
			<TermChips
				terms={terms}
				label={label}
				noun={noun}
				onRemove={(term) => onChange(terms.filter((t) => t !== term))}
			/>
			<div className="flex gap-2">
				<label className="sr-only" htmlFor={id}>
					{inputLabel}
				</label>
				<Input
					id={id}
					value={draft}
					maxLength={maxLength}
					placeholder={placeholder}
					onChange={(event) => setDraft(event.target.value)}
					onKeyDown={(event) => {
						if (event.key === "Enter") {
							// Inside a form this would submit the page instead.
							event.preventDefault();
							add();
						}
					}}
				/>
				<Button
					type="button"
					variant="outline"
					onClick={add}
					disabled={draft.trim().length === 0 || full}
				>
					{addLabel}
				</Button>
			</div>
		</div>
	);
}
