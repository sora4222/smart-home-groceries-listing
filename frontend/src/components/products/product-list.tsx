import { DislikeButton } from "#/components/dislikes/dislike-button";
import { DislikeNotice } from "#/components/dislikes/dislike-notice";
import { ChooseProductButton } from "#/components/products/choose-product-button";
import { PastPurchases } from "#/components/products/past-purchases";
import { ProductRow } from "#/components/products/product-row";
import type { StoreListing } from "#/lib/product-sort";

/**
 * The price comparison's products from both stores in one list, in the
 * order given. Each shows its store, a button to choose it when the
 * comparison can choose, how often it was bought before, a button to
 * dislike it, and a warning when someone dislikes it.
 */
export function ProductList({
	listings,
	quantity,
}: {
	listings: StoreListing[];
	quantity: number;
}) {
	if (listings.length === 0) return null;

	return (
		<ul aria-label="Products" className="divide-y divide-border">
			{listings.map(({ store, storeName, product }) => (
				<li
					key={`${store}:${product.product_id}`}
					data-testid="product-listing"
					data-store={store}
					className="pt-3"
				>
					<p className="text-xs font-semibold text-muted-foreground">
						{storeName}
					</p>
					<ProductRow
						product={product}
						quantity={quantity}
						storeName={storeName}
						notice={<DislikeNotice store={store} product={product} />}
					>
						<DislikeButton
							store={store}
							storeName={storeName}
							product={product}
						/>
						<ChooseProductButton
							store={store}
							storeName={storeName}
							product={product}
						/>
						<PastPurchases
							store={store}
							productId={product.product_id}
							productName={product.name}
						/>
					</ProductRow>
				</li>
			))}
		</ul>
	);
}
