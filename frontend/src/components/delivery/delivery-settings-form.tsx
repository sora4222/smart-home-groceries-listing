import { useState } from "react";

import { AmountField } from "#/components/delivery/amount-field";
import { OrderModeChoice } from "#/components/delivery/order-mode-choice";
import { StoreFeeFields } from "#/components/delivery/store-fee-fields";
import { Button } from "#/components/ui/button";
import type { DeliverySettings } from "#/lib/api";
import { fromForm, toForm } from "#/lib/delivery-form";

/**
 * Settings › Delivery's form: each store's fee rules, the default way to pick
 * stores, and a cap on delivery fees per order.
 *
 * Starts from `settings`; give it a `key` that changes when they do, so a
 * saved or undone value shows. A wrong amount is explained under the form and
 * nothing is sent. `onSave` rejects when saving failed.
 */
export function DeliverySettingsForm({
	settings,
	onSave,
}: {
	settings: DeliverySettings;
	onSave: (settings: DeliverySettings) => Promise<void>;
}) {
	const [form, setForm] = useState(() => toForm(settings));
	const [error, setError] = useState<string | null>(null);
	const [saving, setSaving] = useState(false);

	async function save(event: React.FormEvent) {
		event.preventDefault();
		const parsed = fromForm(form);
		if ("error" in parsed) {
			setError(parsed.error);
			return;
		}
		setError(null);
		setSaving(true);
		try {
			await onSave(parsed.settings);
		} catch {
			// The page has said why.
		} finally {
			setSaving(false);
		}
	}

	return (
		<form
			onSubmit={save}
			aria-label="Delivery settings"
			className="flex flex-col gap-4"
		>
			{form.stores.map((fields, index) => (
				<StoreFeeFields
					key={fields.store}
					fields={fields}
					onChange={(changed) =>
						setForm({
							...form,
							stores: form.stores.map((f, i) => (i === index ? changed : f)),
						})
					}
				/>
			))}

			<OrderModeChoice
				value={form.mode}
				onChange={(mode) => setForm({ ...form, mode })}
			/>

			<div className="sm:max-w-xs">
				<AmountField
					id="max-delivery-spend"
					label="Most to spend on delivery"
					hint="Per order, all stores together. Options over this are not recommended. Empty for no limit."
					value={form.maxDeliverySpend}
					onChange={(maxDeliverySpend) =>
						setForm({ ...form, maxDeliverySpend })
					}
				/>
			</div>

			{error && (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			)}

			<Button
				type="submit"
				disabled={saving}
				className="w-full sm:w-auto sm:self-start"
			>
				{saving ? "Saving…" : "Save"}
			</Button>
		</form>
	);
}
