import type { StoreProduct } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * What a product costs: its shelf price (with the previous price struck
 * through when reduced, unless `showWasPrice` is off), and — when a multibuy
 * applies at the item's quantity — what that many cost with it.
 */
export function ProductPrice({
	product,
	quantity,
	showWasPrice = true,
}: {
	product: StoreProduct;
	quantity: number;
	showWasPrice?: boolean;
}) {
	if (product.price === null) {
		return <p className="text-sm text-muted-foreground">No price</p>;
	}

	return (
		<div className="flex flex-col items-end text-right">
			<p className="font-semibold">
				{formatMoney(product.price)}
				{showWasPrice && product.was_price ? (
					<s className="ml-2 text-xs font-normal text-muted-foreground">
						<span className="sr-only">was </span>
						{formatMoney(product.was_price)}
					</s>
				) : null}
			</p>
			{quantity > 1 && product.total_price && product.deal_applied ? (
				<p className="text-xs text-muted-foreground">
					{quantity} for {formatMoney(product.total_price)} with deal
				</p>
			) : null}
		</div>
	);
}
