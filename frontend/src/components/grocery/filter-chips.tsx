import { X } from "lucide-react";

import { Badge } from "#/components/ui/badge";

interface FilterChipsProps {
	terms: string[];
	/** Omitted when the chips are read-only, as on a committed item. */
	onRemove?: (term: string) => void;
}

/**
 * An item's filter terms, shown as chips (spec: "Item Rules > Runtime
 * behaviour"). A chip narrows the later product search — "3 ply", "organic" —
 * and can be taken off here, on the list, without touching the rule that put
 * it there.
 */
export function FilterChips({ terms, onRemove }: FilterChipsProps) {
	if (terms.length === 0) return null;

	return (
		<ul
			aria-label="Product filters"
			className="flex list-none flex-wrap gap-1 p-0"
		>
			{terms.map((term) => (
				<li key={term}>
					<Badge variant="outline" className="gap-1 pr-1">
						{term}
						{onRemove && (
							<button
								type="button"
								aria-label={`Remove filter ${term}`}
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
