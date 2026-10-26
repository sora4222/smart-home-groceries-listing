import type { ReactNode } from "react";

import { TermChips } from "#/components/terms/term-chips";
import { useEntered } from "#/hooks/useEntered";
import type { GroceryItem } from "#/lib/api";
import { cn } from "#/lib/utils";

/**
 * The Select item page's frame: it slides in from the right, with a way
 * back, the item's name, quantity and chips above the products.
 */
export function SelectItemPage({
	item,
	back,
	children,
}: {
	item: GroceryItem;
	/** The link back to the list. */
	back: ReactNode;
	children: ReactNode;
}) {
	const entered = useEntered();
	return (
		<div
			className={cn(
				"mx-auto flex w-full max-w-3xl flex-col gap-4 transition duration-300 ease-out motion-reduce:transition-none",
				entered ? "translate-x-0 opacity-100" : "translate-x-8 opacity-0",
			)}
		>
			{back}
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">
					Select {item.name} ×{item.quantity}
				</h1>
				<p className="text-sm text-muted-foreground">
					Choose a product to buy as this item.
				</p>
				<TermChips terms={item.filter_terms} />
			</div>
			{children}
		</div>
	);
}
