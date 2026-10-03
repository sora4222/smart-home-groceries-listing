/**
 * The short warnings an order option carries, in words. Pure.
 *
 * An option can still be shown when one of these applies; the notes say why
 * it is not recommended, or what to check first.
 */
import type { OptionStore, OrderOption } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/** One store's delivery, as a phrase: "$9.00 delivery", "free delivery". */
export function deliveryPhrase(store: OptionStore): string {
	if (store.free_delivery) return "free delivery";
	if (!store.fee_known) return "delivery fee not set";
	return `${formatMoney(store.delivery_fee)} delivery`;
}

/** Every warning for `option`, most important first. */
export function optionNotes(
	option: OrderOption,
	maxDeliverySpend: string | null,
): string[] {
	const notes: string[] = [];
	if (!option.complete) {
		const count = option.missing.length;
		notes.push(`Leaves out ${count} ${count === 1 ? "item" : "items"}.`);
	}
	for (const store of option.stores) {
		if (store.below_minimum && store.minimum_order) {
			notes.push(
				`Under the ${store.store_name} minimum order of ${formatMoney(store.minimum_order)}.`,
			);
		}
	}
	if (!option.within_delivery_cap && maxDeliverySpend) {
		notes.push(`Delivery is over your ${formatMoney(maxDeliverySpend)} limit.`);
	}
	if (!option.fees_known) {
		notes.push("Some delivery fees are not set, so they count as $0.");
	}
	return notes;
}
