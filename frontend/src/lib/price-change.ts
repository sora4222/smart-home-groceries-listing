/**
 * Plain words for how an order line's price moved since it was chosen.
 */
import type { OrderLine } from "#/lib/api/order-review";
import { formatMoney } from "#/lib/money";

/**
 * `"Up from $2.80"` / `"Down from $5.20"`, or `null` when the price is the
 * same or either price is unknown.
 */
export function priceChangeText(
	line: Pick<OrderLine, "price_change" | "chosen_price">,
): string | null {
	if (!line.chosen_price) return null;
	const was = formatMoney(line.chosen_price);
	if (line.price_change === "up") return `Up from ${was}`;
	if (line.price_change === "down") return `Down from ${was}`;
	return null;
}
