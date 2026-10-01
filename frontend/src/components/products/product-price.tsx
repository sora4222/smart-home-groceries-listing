import type { StoreProduct } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * What a product costs: its shelf price (with the previous price struck
 * through when reduced), and — when the item wants more than one — what
 * that many cost with the best multibuy applied.
 */
export function ProductPrice({
	product,
	quantity,
}: {
	product: StoreProduct;
	quantity: number;
}) {
	if (product.price === null) {
		return <p className="text-sm text-muted-foreground">No price</p>;
	}

	return (
		<div className="flex flex-col items-end text-right">
			<p className="font-semibold">
				{formatMoney(product.price)}
				{product.was_price ? (
					<s className="ml-2 text-xs font-normal text-muted-foreground">
						<span className="sr-only">was </span>
						{formatMoney(product.was_price)}
					</s>
				) : null}
			</p>
			{quantity > 1 && product.total_price ? (
				<p className="text-xs text-muted-foreground">
					{quantity} for {formatMoney(product.total_price)}
					{product.deal_applied ? " with deal" : ""}
				</p>
			) : null}
		</div>
	);
}
