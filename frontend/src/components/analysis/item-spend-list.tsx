import { useState } from "react";

import { ItemPriceHistory } from "#/components/analysis/item-price-history";
import { SpendBars } from "#/components/analysis/spend-bars";
import { Button } from "#/components/ui/button";
import type { ItemSpend } from "#/lib/api";

/**
 * "By item": spend per list item, biggest first. Each item opens to the
 * price paid on every shop.
 */
export function ItemSpendList({ items }: { items: ItemSpend[] }) {
	const [open, setOpen] = useState<string | null>(null);
	return (
		<SpendBars
			label="Spend by item"
			empty="Nothing bought in this range."
			bars={items.map((item) => ({
				key: item.item_name,
				label: item.item_name,
				amount: item.total,
				detail: `${times(item.times_bought)} · ${item.quantity} bought`,
			}))}
			renderExtra={(bar) => (
				<div className="flex flex-col gap-1">
					<Button
						type="button"
						size="sm"
						variant="ghost"
						className="h-auto self-start px-0 py-0 text-xs text-muted-foreground"
						aria-expanded={open === bar.key}
						aria-label={`${open === bar.key ? "Hide" : "Show"} prices paid for ${bar.label}`}
						onClick={() => setOpen(open === bar.key ? null : bar.key)}
					>
						{open === bar.key ? "Hide prices" : "Prices paid"}
					</Button>
					{open === bar.key && <ItemPriceHistory itemName={bar.label} />}
				</div>
			)}
		/>
	);
}

/** "once", "2 times". */
function times(count: number): string {
	return count === 1 ? "once" : `${count} times`;
}
