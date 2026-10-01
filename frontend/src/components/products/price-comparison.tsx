import { createContext, type ReactNode, useContext, useState } from "react";

import { SpecialsOnlySwitch } from "#/components/products/specials-only-switch";
import { StoreResults } from "#/components/products/store-results";
import { TermChips } from "#/components/terms/term-chips";
import { Button } from "#/components/ui/button";
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "#/components/ui/sheet";
import { useItemProducts } from "#/hooks/useItemProducts";
import type { GroceryItem } from "#/lib/api";

/** The item being compared, shared by the parts of one comparison only. */
const ItemContext = createContext<GroceryItem | null>(null);

function useComparedItem(): GroceryItem {
	const item = useContext(ItemContext);
	if (!item)
		throw new Error("PriceComparison parts must be inside <PriceComparison>");
	return item;
}

/**
 * Woolworths and Coles side by side for one list item, in a sheet.
 *
 * ```tsx
 * <PriceComparison item={item}>
 *   <PriceComparison.Trigger />
 *   <PriceComparison.Content />
 * </PriceComparison>
 * ```
 *
 * The stores are searched only once the sheet opens.
 */
function PriceComparisonRoot({
	item,
	children,
}: {
	item: GroceryItem;
	children: ReactNode;
}) {
	return (
		<ItemContext.Provider value={item}>
			<Sheet>{children}</Sheet>
		</ItemContext.Provider>
	);
}

/** The button that opens the comparison. */
function Trigger() {
	const item = useComparedItem();
	return (
		<SheetTrigger asChild>
			<Button
				size="sm"
				variant="outline"
				aria-label={`Compare prices for ${item.name}`}
			>
				Compare prices
			</Button>
		</SheetTrigger>
	);
}

/** The sheet: the item, the specials filter, and each store's products. */
function Content() {
	const item = useComparedItem();
	return (
		<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
			<SheetHeader>
				<SheetTitle>
					Prices for {item.name} ×{item.quantity}
				</SheetTitle>
				<SheetDescription>
					Cheapest per unit first, at Woolworths and Coles.
				</SheetDescription>
				<TermChips terms={item.filter_terms} />
			</SheetHeader>
			{/* Mounted only while the sheet is open, so the search runs then. */}
			<Results item={item} />
		</SheetContent>
	);
}

/** Runs the search and shows its progress or its results. */
function Results({ item }: { item: GroceryItem }) {
	const [specialsOnly, setSpecialsOnly] = useState(false);
	const searchKey = `${item.name}|${item.quantity}|${item.filter_terms.join(",")}`;
	const { state, retry } = useItemProducts(item.id, true, searchKey);

	if (state.status === "idle" || state.status === "loading") {
		return (
			<output className="block px-4 text-sm text-muted-foreground">
				Searching the stores…
			</output>
		);
	}

	if (state.status === "failed") {
		return (
			<div role="alert" className="flex flex-col items-start gap-2 px-4">
				<p className="text-sm">The stores could not be searched.</p>
				<Button size="sm" variant="outline" onClick={retry}>
					Try again
				</Button>
			</div>
		);
	}

	return (
		<div className="flex flex-col gap-4 px-4 pb-4">
			<SpecialsOnlySwitch
				checked={specialsOnly}
				onCheckedChange={setSpecialsOnly}
			/>
			{state.data.stores.map((results) => (
				<StoreResults
					key={results.store}
					results={results}
					quantity={state.data.item.quantity}
					specialsOnly={specialsOnly}
				/>
			))}
		</div>
	);
}

export const PriceComparison = Object.assign(PriceComparisonRoot, {
	Trigger,
	Content,
});
