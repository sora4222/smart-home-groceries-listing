/**
 * Formatting the backend's decimal-string prices for people.
 *
 * Prices stay strings until they are shown; nothing here does arithmetic,
 * so nothing is rounded except for display.
 */
import type { UnitPrice } from "#/lib/api/products";

/** `"4.5"` → `"$4.50"`. An unreadable value is shown as it came. */
export function formatMoney(amount: string): string {
	const value = Number(amount);
	if (!Number.isFinite(value)) return amount;
	return `$${value.toFixed(2)}`;
}

/**
 * A unit price with as many decimals as it needs to be compared: two for a
 * dollar or more, three for cents, four for fractions of a cent (one sheet
 * of toilet paper). Trailing zeros past the second decimal are dropped.
 */
export function formatUnitAmount(amount: string): string {
	const value = Number(amount);
	if (!Number.isFinite(value)) return amount;
	const places = value >= 1 ? 2 : value >= 0.01 ? 3 : 4;
	const fixed = value.toFixed(places);
	const trimmed = fixed.replace(/(\.\d\d\d*?)0+$/, "$1");
	return `$${trimmed}`;
}

/** `{ amount: "0.155", per: "100mL" }` → `"$0.155 / 100mL"`. */
export function formatUnitPrice(unit: UnitPrice): string {
	return `${formatUnitAmount(unit.amount)} / ${unit.per}`;
}
