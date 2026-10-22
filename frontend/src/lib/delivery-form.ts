/**
 * Settings › Delivery's form values, to and from the API's shape.
 *
 * Inputs hold strings; an empty input means "not set" (`null`). An amount is
 * $0 to $1000 in whole cents, the same limits the backend checks. Pure.
 */
import type {
	DeliverySettings,
	OrderMode,
	StoreFeeRules,
	StoreId,
} from "#/lib/api";

/** The editable fields of one store. */
export interface StoreFeeFields {
	store: StoreId;
	storeName: string;
	deliveryFee: string;
	freeDeliveryOver: string;
	minimumOrder: string;
}

/** The whole form. */
export interface DeliveryForm {
	stores: StoreFeeFields[];
	mode: OrderMode;
	maxDeliverySpend: string;
}

/** The settings as form values. */
export function toForm(settings: DeliverySettings): DeliveryForm {
	return {
		stores: settings.stores.map((rules) => ({
			store: rules.store,
			storeName: rules.store_name ?? rules.store,
			deliveryFee: rules.delivery_fee ?? "",
			freeDeliveryOver: rules.free_delivery_over ?? "",
			minimumOrder: rules.minimum_order ?? "",
		})),
		mode: settings.mode,
		maxDeliverySpend: settings.max_delivery_spend ?? "",
	};
}

/** An amount input: `""` → `null`, `"$9.5"` → `"9.5"`, nonsense → error. */
export function parseAmount(
	input: string,
): { ok: true; value: string | null } | { ok: false } {
	const trimmed = input.trim().replace(/^\$/, "");
	if (trimmed === "") return { ok: true, value: null };
	if (!/^\d{1,4}(\.\d{1,2})?$/.test(trimmed)) return { ok: false };
	if (Number(trimmed) > 1000) return { ok: false };
	return { ok: true, value: trimmed };
}

/**
 * The form as settings to save, or the sentence explaining the first
 * amount that is not one.
 */
export function fromForm(
	form: DeliveryForm,
): { settings: DeliverySettings } | { error: string } {
	const stores: StoreFeeRules[] = [];
	for (const fields of form.stores) {
		const fee = parseAmount(fields.deliveryFee);
		const freeOver = parseAmount(fields.freeDeliveryOver);
		const minimum = parseAmount(fields.minimumOrder);
		if (!fee.ok || !freeOver.ok || !minimum.ok) {
			return { error: amountError(`${fields.storeName}'s amounts`) };
		}
		stores.push({
			store: fields.store,
			delivery_fee: fee.value,
			free_delivery_over: freeOver.value,
			minimum_order: minimum.value,
		});
	}
	const cap = parseAmount(form.maxDeliverySpend);
	if (!cap.ok) return { error: amountError("The delivery spending limit") };
	return {
		settings: { stores, mode: form.mode, max_delivery_spend: cap.value },
	};
}

function amountError(what: string): string {
	return `${what} must be dollars and cents from 0 to 1000, like 9.00, or empty.`;
}
