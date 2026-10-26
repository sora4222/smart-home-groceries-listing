import { type ReactNode, useState } from "react";

import { ProductDislikesProvider } from "#/components/dislikes/product-dislikes";
import { PastPurchasesProvider } from "#/components/products/past-purchases";
import { ProductFilterBar } from "#/components/products/product-filter-bar";
import { ProductList } from "#/components/products/product-list";
import { ProductSortSelect } from "#/components/products/product-sort-select";
import { SpecialsOnlySwitch } from "#/components/products/specials-only-switch";
import { StoreNotices } from "#/components/products/store-notices";
import { Button } from "#/components/ui/button";
import { useDisplaySettings } from "#/hooks/useDisplaySettings";
import { useItemDislikes } from "#/hooks/useItemDislikes";
import { useItemProducts } from "#/hooks/useItemProducts";
import { useProductHistory } from "#/hooks/useProductHistory";
import type { GroceryItem, ItemProducts } from "#/lib/api";
import {
	NO_FILTERS,
	type ProductFilters,
	shownListings,
	storesShown,
} from "#/lib/product-filters";
import {
	commonBasis,
	DEFAULT_SORT,
	type ProductSort,
	sortListings,
} from "#/lib/product-sort";

/**
 * Searches both stores for one list item and shows the products in one
 * list, with sort, filters and the display settings applied. Mounting it
 * starts the search.
 */
export function ProductResults({ item }: { item: GroceryItem }) {
	const dislikes = useItemDislikes(item.id);
	const display = useDisplaySettings();
	const [sort, setSort] = useState<ProductSort>(DEFAULT_SORT);
	const [filters, setFilters] = useState<ProductFilters>(NO_FILTERS);
	const { state, retry } = useItemProducts(item);

	if (state.status === "loading") {
		return (
			<output className="block text-sm text-muted-foreground">
				Searching the stores…
			</output>
		);
	}

	if (state.status === "failed") {
		return (
			<div role="alert" className="flex flex-col items-start gap-2">
				<p className="text-sm">The stores could not be searched.</p>
				<Button size="sm" variant="outline" onClick={retry}>
					Try again
				</Button>
			</div>
		);
	}

	const listings = shownListings(
		state.data.stores,
		filters,
		display.showUnavailable,
	);

	return (
		<ProductDislikesProvider value={{ itemId: item.id, ...dislikes }}>
			<div className="flex flex-col gap-4">
				<div className="flex flex-wrap items-center justify-between gap-2">
					<ProductSortSelect value={sort} onValueChange={setSort} />
					<SpecialsOnlySwitch
						checked={filters.specialsOnly}
						onCheckedChange={(specialsOnly) =>
							setFilters({ ...filters, specialsOnly })
						}
					/>
				</div>
				<ProductFilterBar value={filters} onChange={setFilters} />
				<StoreNotices
					stores={storesShown(state.data.stores, filters)}
					specialsOnly={filters.specialsOnly}
				/>
				<WithPastPurchases search={state.data}>
					<ProductList
						listings={sortListings(listings, sort, {
							quantity: state.data.item.quantity,
							basis: commonBasis(
								state.data.stores.flatMap((results) => results.products),
							),
						})}
						quantity={state.data.item.quantity}
						display={display}
					/>
				</WithPastPurchases>
			</div>
		</ProductDislikesProvider>
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
