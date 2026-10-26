import { createContext, type ReactNode, useContext, useState } from "react";

import { ProductDislikesProvider } from "#/components/dislikes/product-dislikes";
import { PastPurchasesProvider } from "#/components/products/past-purchases";
import { ProductChoiceProvider } from "#/components/products/product-choice";
import { ProductFilterBar } from "#/components/products/product-filter-bar";
import { ProductList } from "#/components/products/product-list";
import { ProductSortSelect } from "#/components/products/product-sort-select";
import { SpecialsOnlySwitch } from "#/components/products/specials-only-switch";
import { StoreNotices } from "#/components/products/store-notices";
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
import { useItemDislikes } from "#/hooks/useItemDislikes";
import { useItemProducts } from "#/hooks/useItemProducts";
import { useProductHistory } from "#/hooks/useProductHistory";
import type {
	GroceryItem,
	ItemProducts,
	ProductChoice,
	StoreProducts,
} from "#/lib/api";
import {
	filterListings,
	NO_FILTERS,
	type ProductFilters,
} from "#/lib/product-filters";
import {
	commonBasis,
	DEFAULT_SORT,
	type ProductSort,
	type StoreListing,
	sortListings,
} from "#/lib/product-sort";
import { visibleProducts } from "#/lib/specials";

/** The item being compared, shared by the parts of one comparison only. */
const ItemContext = createContext<GroceryItem | null>(null);

function useComparedItem(): GroceryItem {
	const item = useContext(ItemContext);
	if (!item)
		throw new Error("PriceComparison parts must be inside <PriceComparison>");
	return item;
}

/**
 * Woolworths and Coles in one sortable list for one list item, in a sheet.
 *
 * ```tsx
 * <PriceComparison item={item}>
 *   <PriceComparison.Trigger />
 *   <PriceComparison.Content />
 * </PriceComparison>
 * ```
 *
 * The stores are searched only once the sheet opens. Pass `onChoose` to let
 * the household choose the item's product from the sheet; `chosen` marks the
 * products chosen already (one per store at most). Without `onChoose` the sheet only compares.
 * Every product can be disliked, and a disliked one shows who disliked it.
 */
function PriceComparisonRoot({
	item,
	chosen = [],
	onChoose,
	children,
}: {
	item: GroceryItem;
	chosen?: ProductChoice[];
	onChoose?: (choice: ProductChoice) => Promise<void>;
	children: ReactNode;
}) {
	const sheet = <Sheet>{children}</Sheet>;
	return (
		<ItemContext.Provider value={item}>
			{onChoose ? (
				<ProductChoiceProvider value={{ chosen, onChoose }}>
					{sheet}
				</ProductChoiceProvider>
			) : (
				sheet
			)}
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

/** The sheet: the item, the sort and specials filter, and the products. */
function Content() {
	const item = useComparedItem();
	return (
		<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
			<SheetHeader>
				<SheetTitle>
					Prices for {item.name} ×{item.quantity}
				</SheetTitle>
				<SheetDescription>
					Woolworths and Coles together, cheapest per unit first unless you sort
					them another way. Choose one product to buy for this item.
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
	const dislikes = useItemDislikes(item.id);
	const [specialsOnly, setSpecialsOnly] = useState(false);
	const [sort, setSort] = useState<ProductSort>(DEFAULT_SORT);
	const [filters, setFilters] = useState<ProductFilters>(NO_FILTERS);
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
		<ProductDislikesProvider value={{ itemId: item.id, ...dislikes }}>
			<div className="flex flex-col gap-4 px-4 pb-4">
				<div className="flex flex-wrap items-center justify-between gap-2">
					<ProductSortSelect value={sort} onValueChange={setSort} />
					<SpecialsOnlySwitch
						checked={specialsOnly}
						onCheckedChange={setSpecialsOnly}
					/>
				</div>
				<ProductFilterBar value={filters} onChange={setFilters} />
				<StoreNotices stores={state.data.stores} specialsOnly={specialsOnly} />
				<WithPastPurchases search={state.data}>
					<ProductList
						listings={sortListings(
							filterListings(
								listingsOf(state.data.stores, specialsOnly),
								filters,
							),
							sort,
							{
								quantity: state.data.item.quantity,
								basis: commonBasis(
									state.data.stores.flatMap((results) => results.products),
								),
							},
						)}
						quantity={state.data.item.quantity}
					/>
				</WithPastPurchases>
			</div>
		</ProductDislikesProvider>
	);
}

/** Every product shown from the stores that answered, as one list. */
function listingsOf(
	stores: StoreProducts[],
	specialsOnly: boolean,
): StoreListing[] {
	return stores
		.filter((results) => results.status === "ok")
		.flatMap((results) =>
			visibleProducts(results.products, specialsOnly).map((product) => ({
				store: results.store,
				storeName: results.store_name,
				product,
			})),
		);
}

/** Reads the past purchases of every product found, for their rows. */
function WithPastPurchases({
	search,
	children,
}: {
	search: ItemProducts;
	children: ReactNode;
}) {
	const history = useProductHistory(
		search.stores.flatMap((results) =>
			results.products.map((product) => ({
				store: results.store,
				product_id: product.product_id,
			})),
		),
	);
	return (
		<PastPurchasesProvider history={history}>{children}</PastPurchasesProvider>
	);
}

export const PriceComparison = Object.assign(PriceComparisonRoot, {
	Trigger,
	Content,
});
