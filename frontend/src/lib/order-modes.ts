/**
 * The order planner's modes, with the words a person reads for each.
 * Shared by Settings › Delivery and the order screen.
 */
import type { OrderMode } from "#/lib/api";

/** A mode and how it is described. */
export interface OrderModeInfo {
	mode: OrderMode;
	label: string;
	description: string;
}

/** Every mode, in the order they are offered. */
export const ORDER_MODES: OrderModeInfo[] = [
	{
		mode: "minimise_total",
		label: "Cheapest total",
		description:
			"Lowest items plus delivery. Splits the order between stores when that is cheaper.",
	},
	{
		mode: "minimise_delivery",
		label: "Cheapest delivery",
		description: "Lowest delivery fees first, then the lowest items.",
	},
	{
		mode: "woolworths_only",
		label: "Woolworths only",
		description: "Everything from Woolworths.",
	},
	{
		mode: "coles_only",
		label: "Coles only",
		description: "Everything from Coles.",
	},
	{
		mode: "manual",
		label: "I pick the store",
		description:
			"You choose each item's store on the order screen; the app adds it up.",
	},
];

/** The label for `mode`. */
export function orderModeLabel(mode: OrderMode): string {
	return ORDER_MODES.find((m) => m.mode === mode)?.label ?? mode;
}
