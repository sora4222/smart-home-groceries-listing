import { useId } from "react";

import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
import { type ProductSort, SORT_CHOICES } from "#/lib/product-sort";

/** "Sort by": the order the comparison's products are listed in. */
export function ProductSortSelect({
	value,
	onValueChange,
}: {
	value: ProductSort;
	onValueChange: (value: ProductSort) => void;
}) {
	const id = useId();
	return (
		<div className="flex items-center gap-2">
			<label htmlFor={id} className="text-sm">
				Sort by
			</label>
			<Select
				value={value}
				onValueChange={(next) => onValueChange(next as ProductSort)}
			>
				<SelectTrigger id={id} className="w-56">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{SORT_CHOICES.map((choice) => (
						<SelectItem key={choice.value} value={choice.value}>
							{choice.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	);
}
