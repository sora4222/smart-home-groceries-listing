import type { StoreProduct } from "#/lib/api";
import { formatUnitPrice } from "#/lib/money";

/**
 * A product's unit price, the thing products are compared on, with the
 * backend's note when it needs care: "unit price calculated from 1kg",
 * "units differ — compare by hand", "no unit price from the store".
 */
export function UnitPriceLine({ product }: { product: StoreProduct }) {
	return (
		<p className="text-sm">
			{product.unit_price ? (
				<span className="font-medium">
					{formatUnitPrice(product.unit_price)}
				</span>
			) : null}
			{product.unit_price_note ? (
				<span className="text-xs text-muted-foreground">
					{product.unit_price ? " · " : ""}
					{product.unit_price_note}
				</span>
			) : null}
		</p>
	);
}
