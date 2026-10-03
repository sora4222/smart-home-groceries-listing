import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";

/**
 * One dollar amount on Settings › Delivery: a label, a `$` input and a hint.
 * Empty means "not set"; the form checks the value when it is saved.
 */
export function AmountField({
	id,
	label,
	hint,
	value,
	onChange,
}: {
	id: string;
	label: string;
	hint: string;
	value: string;
	onChange: (value: string) => void;
}) {
	return (
		<div className="flex flex-col gap-1">
			<Label htmlFor={id}>{label}</Label>
			<div className="flex items-center gap-1">
				<span aria-hidden className="text-sm text-muted-foreground">
					$
				</span>
				<Input
					id={id}
					inputMode="decimal"
					autoComplete="off"
					value={value}
					placeholder="Not set"
					onChange={(event) => onChange(event.target.value)}
					aria-describedby={`${id}-hint`}
				/>
			</div>
			<p id={`${id}-hint`} className="text-xs text-muted-foreground">
				{hint}
			</p>
		</div>
	);
}
