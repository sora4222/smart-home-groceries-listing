import { AmountField } from "#/components/delivery/amount-field";
import type { StoreFeeFields as Fields } from "#/lib/delivery-form";

/** One store's delivery fee, free-delivery amount and minimum order. */
export function StoreFeeFields({
	fields,
	onChange,
}: {
	fields: Fields;
	onChange: (fields: Fields) => void;
}) {
	const id = (name: string) => `${fields.store}-${name}`;
	return (
		<fieldset className="flex flex-col gap-3 rounded-md border border-border p-3">
			<legend className="px-1 text-sm font-medium">{fields.storeName}</legend>
			<div className="grid gap-3 sm:grid-cols-3">
				<AmountField
					id={id("fee")}
					label="Delivery fee"
					hint="What one delivery costs."
					value={fields.deliveryFee}
					onChange={(deliveryFee) => onChange({ ...fields, deliveryFee })}
				/>
				<AmountField
					id={id("free-over")}
					label="Free delivery from"
					hint="Orders this big or bigger deliver free. Empty if never."
					value={fields.freeDeliveryOver}
					onChange={(freeDeliveryOver) =>
						onChange({ ...fields, freeDeliveryOver })
					}
				/>
				<AmountField
					id={id("minimum")}
					label="Minimum order"
					hint="The store will not deliver a smaller order. Empty if none."
					value={fields.minimumOrder}
					onChange={(minimumOrder) => onChange({ ...fields, minimumOrder })}
				/>
			</div>
		</fieldset>
	);
}
