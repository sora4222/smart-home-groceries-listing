import { ProductRow } from "#/components/products/product-row";
import type { StoreProducts } from "#/lib/api";
import { visibleProducts } from "#/lib/specials";

/**
 * One store's part of the price comparison: its products cheapest per unit
 * first, or why there are none — the store failed, nothing matched the
 * item's filters, or nothing is on special.
 */
export function StoreResults({
	results,
	quantity,
	specialsOnly,
}: {
	results: StoreProducts;
	quantity: number;
	specialsOnly: boolean;
}) {
	const headingId = `store-${results.store}`;
	const products = visibleProducts(results.products, specialsOnly);

	return (
		<section
			aria-labelledby={headingId}
			data-testid="store-results"
			data-store={results.store}
		>
			<h3 id={headingId} className="text-sm font-semibold">
				{results.store_name}
			</h3>

			{results.status !== "ok" ? (
				<output className="block py-2 text-sm text-muted-foreground">
					{results.message ?? `${results.store_name} could not be searched.`}
				</output>
			) : products.length === 0 ? (
				<p className="py-2 text-sm text-muted-foreground">
					{emptyMessage(results.products.length > 0, results.store_name)}
				</p>
			) : (
				<div className="divide-y divide-border">
					{products.map((product) => (
						<ProductRow
							key={product.product_id}
							product={product}
							quantity={quantity}
							storeName={results.store_name}
						/>
					))}
				</div>
			)}
		</section>
	);
}

/** Why a store that answered shows nothing. */
function emptyMessage(hasProducts: boolean, storeName: string): string {
	return hasProducts
		? `Nothing on special at ${storeName}.`
		: `No products at ${storeName} match this item and its filters.`;
}
