import { Label } from "#/components/ui/label";
import { RadioGroup, RadioGroupItem } from "#/components/ui/radio-group";
import type { OrderMode } from "#/lib/api";
import { ORDER_MODES } from "#/lib/order-modes";

/** The order planner's default mode: one choice from `ORDER_MODES`. */
export function OrderModeChoice({
	value,
	onChange,
}: {
	value: OrderMode;
	onChange: (mode: OrderMode) => void;
}) {
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">How to pick stores</legend>
			<RadioGroup
				value={value}
				onValueChange={(mode) => onChange(mode as OrderMode)}
				className="gap-3"
			>
				{ORDER_MODES.map((info) => (
					<div key={info.mode} className="flex items-start gap-2">
						<RadioGroupItem
							id={`mode-${info.mode}`}
							value={info.mode}
							aria-describedby={`mode-${info.mode}-hint`}
						/>
						<div className="flex flex-col gap-0.5">
							<Label htmlFor={`mode-${info.mode}`}>{info.label}</Label>
							<p
								id={`mode-${info.mode}-hint`}
								className="text-xs text-muted-foreground"
							>
								{info.description}
							</p>
						</div>
					</div>
				))}
			</RadioGroup>
		</fieldset>
	);
}
