import type { ReactNode } from "react";

import { GroceryItemCard } from "#/components/grocery/grocery-item-card";

interface GroceryListProps {
	children: ReactNode;
}

/**
 * The list container. A compound component: the page composes
 * `GroceryList` with `GroceryList.Item` and `GroceryList.Empty` rather than
 * passing a render-prop or an array of options down, so a new section (the
 * committed items, say) is a matter of composition at the call site.
 */
export function GroceryList({ children }: GroceryListProps) {
	return <ul className="flex list-none flex-col gap-2 p-0">{children}</ul>;
}

/** One item on the list. */
GroceryList.Item = function Item({ children }: { children: ReactNode }) {
	return <li>{children}</li>;
};

/** Shown in place of the items when there are none. */
GroceryList.Empty = function Empty({ children }: { children: ReactNode }) {
	return (
		<li className="rounded-md border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
			{children}
		</li>
	);
};

export { GroceryItemCard };
