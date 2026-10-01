import { Badge } from "#/components/ui/badge";
import type { OrderLine } from "#/lib/api";
import { priceChangeText } from "#/lib/price-change";

/**
 * "Up from $2.80" or "Down from $5.20" when the shelf price moved since the
 * product was chosen; nothing otherwise.
 */
export function PriceChangeBadge({
	line,
}: {
	line: Pick<OrderLine, "price_change" | "chosen_price">;
}) {
	const text = priceChangeText(line);
	if (!text) return null;
	return (
		<Badge variant={line.price_change === "up" ? "destructive" : "outline"}>
			{text}
		</Badge>
	);
}
