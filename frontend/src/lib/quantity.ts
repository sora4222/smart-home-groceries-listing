/** The item quantity bounds, matching the `quantity` column's CHECK constraint. */
export const MIN_QUANTITY = 1;
export const MAX_QUANTITY = 999;

/**
 * Reads a quantity field as a whole number inside the column's range.
 *
 * Quantity inputs hold what the user typed rather than a number, so an emptied
 * field does not snap back to 1 and make the next digit append to it. This is
 * where that string becomes a number, on blur and on submit.
 */
export function clampQuantity(typed: string): number {
	const parsed = Number.parseInt(typed, 10);
	if (Number.isNaN(parsed)) return MIN_QUANTITY;
	return Math.min(MAX_QUANTITY, Math.max(MIN_QUANTITY, parsed));
}
