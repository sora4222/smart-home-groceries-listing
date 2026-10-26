import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";

/**
 * How many of one product to buy, when an item has several products. Offers
 * 1 to `max`, where `max` leaves at least one for each other product.
 */
export function CountSelect({
	label,
	value,
	max,
	onChange,
}: {
	label: string;
	value: number;
	max: number;
	onChange: (count: number) => void;
}) {
	const counts = Array.from({ length: Math.max(max, value) }, (_, i) => i + 1);
	return (
		<Select
			value={String(value)}
			onValueChange={(count) => onChange(Number(count))}
		>
			<SelectTrigger aria-label={label} size="sm" className="w-20">
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				{counts.map((count) => (
					<SelectItem key={count} value={String(count)}>
						×{count}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
