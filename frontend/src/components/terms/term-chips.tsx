import { X } from "lucide-react";

import { Badge } from "#/components/ui/badge";

interface TermChipsProps {
	terms: readonly string[];
	/** The list's accessible name, e.g. "Product filters". */
	label?: string;
	/** What one chip is, for its remove button: "Remove filter 3 ply". */
	noun?: string;
	/** Omitted when the chips are read-only, as on a committed item. */
	onRemove?: (term: string) => void;
}

/**
 * A list of short terms shown as chips, each optionally removable.
 *
 * On a grocery item these are its filter terms (spec: "Item Rules > Runtime
 * behaviour"), which can be taken off on the list without touching the rule
 * that put them there. On an item rule they are its triggers and its filters.
 */
export function TermChips({
	terms,
	label = "Product filters",
	noun = "filter",
	onRemove,
}: TermChipsProps) {
	if (terms.length === 0) return null;

	return (
		<ul aria-label={label} className="flex list-none flex-wrap gap-1 p-0">
			{terms.map((term) => (
				<li key={term}>
					<Badge variant="outline" className="gap-1 pr-1">
						{term}
						{onRemove && (
							<button
								type="button"
								aria-label={`Remove ${noun} ${term}`}
								onClick={() => onRemove(term)}
								className="rounded-full p-0.5 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
							>
								<X aria-hidden className="size-3" />
							</button>
						)}
					</Badge>
				</li>
			))}
		</ul>
	);
}
