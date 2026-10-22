import { Button } from "#/components/ui/button";
import type { OrderMode } from "#/lib/api";
import { ORDER_MODES } from "#/lib/order-modes";

/**
 * Which way the order screen ranks its options, for this visit only. The
 * saved default is on Settings › Delivery.
 */
export function ModePicker({
	value,
	onChange,
}: {
	value: OrderMode;
	onChange: (mode: OrderMode) => void;
}) {
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">Rank by</legend>
			<div className="flex flex-wrap gap-2">
				{ORDER_MODES.map((info) => (
					<Button
						key={info.mode}
						type="button"
						size="sm"
						variant={info.mode === value ? "secondary" : "outline"}
						aria-pressed={info.mode === value}
						title={info.description}
						onClick={() => onChange(info.mode)}
					>
						{info.label}
					</Button>
				))}
			</div>
		</fieldset>
	);
}
