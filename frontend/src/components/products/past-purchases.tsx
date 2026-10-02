import { createContext, type ReactNode, useContext, useState } from "react";

import { Button } from "#/components/ui/button";
import type { ProductHistory, StoreId } from "#/lib/api";
import { productKey } from "#/lib/api";
import { dayLabel } from "#/lib/day-label";
import { formatMoney } from "#/lib/money";

const HistoryContext = createContext<Map<string, ProductHistory>>(new Map());

/**
 * Makes the past purchases of a comparison's products available to its
 * rows. Scoped to one comparison — never global.
 */
export function PastPurchasesProvider({
	history,
	children,
}: {
	history: Map<string, ProductHistory>;
	children: ReactNode;
}) {
	return (
		<HistoryContext.Provider value={history}>
			{children}
		</HistoryContext.Provider>
	);
}

/**
 * "Bought 3 times" for a product the household bought before, opening to
 * the price paid each time (newest first). Nothing for a product never
 * bought. Only facts about past shops — it never suggests a product.
 */
export function PastPurchases({
	store,
	productId,
	productName,
}: {
	store: StoreId;
	productId: string;
	productName: string;
}) {
	const history = useContext(HistoryContext).get(productKey(store, productId));
	const [open, setOpen] = useState(false);
	if (!history) return null;

	const times = history.times_bought;
	const listId = `past-${store}-${productId}`;
	return (
		<div className="w-full">
			<Button
				type="button"
				size="sm"
				variant="ghost"
				className="h-auto px-0 py-0 text-xs text-muted-foreground"
				aria-expanded={open}
				aria-controls={listId}
				aria-label={`Bought ${productName} ${times === 1 ? "once" : `${times} times`} — ${open ? "hide" : "show"} prices paid`}
				onClick={() => setOpen((value) => !value)}
			>
				Bought {times === 1 ? "once" : `${times} times`}
			</Button>
			{open && (
				<ul id={listId} className="mt-1 flex flex-col gap-0.5 text-xs">
					{history.purchases.map((purchase) => (
						<li
							key={`${purchase.bought_at}-${purchase.quantity}-${purchase.unit_price}`}
							className="flex justify-between gap-2 text-muted-foreground"
						>
							<span>{dayLabel(purchase.bought_at)}</span>
							<span>
								{formatMoney(purchase.unit_price)} each × {purchase.quantity}
							</span>
						</li>
					))}
				</ul>
			)}
		</div>
	);
}
