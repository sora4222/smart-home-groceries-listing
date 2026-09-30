import type { ReactNode } from "react";

/**
 * The rules container. A compound component, like `GroceryList`: the page
 * composes `ItemRuleList` with `ItemRuleList.Item` and `ItemRuleList.Empty`.
 */
export function ItemRuleList({ children }: { children: ReactNode }) {
	return (
		<ul aria-label="Item rules" className="flex list-none flex-col gap-2 p-0">
			{children}
		</ul>
	);
}

/** One rule. */
ItemRuleList.Item = function Item({ children }: { children: ReactNode }) {
	return <li>{children}</li>;
};

/** Shown in place of the rules when there are none. */
ItemRuleList.Empty = function Empty({ children }: { children: ReactNode }) {
	return (
		<li className="rounded-md border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
			{children}
		</li>
	);
};
