import { useId, useState } from "react";

import { Input } from "#/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
import type { ProductFilters } from "#/lib/product-filters";

const STORES = [
	{ value: "both", label: "Both stores" },
	{ value: "woolworths", label: "Woolworths" },
	{ value: "coles", label: "Coles" },
] as const;

const MEASURES = [
	{ value: "any", label: "Any size" },
	{ value: "grams", label: "Grams" },
	{ value: "litres", label: "Litres" },
] as const;

/** Store, size measure and highest price: what the product list shows. */
export function ProductFilterBar({
	value,
	onChange,
}: {
	value: ProductFilters;
	onChange: (next: ProductFilters) => void;
}) {
	const id = useId();
	// What was typed, kept as text so an emptied field stays empty.
	const [price, setPrice] = useState(value.maxPrice?.toString() ?? "");

	function changePrice(text: string) {
		setPrice(text);
		const amount = Number(text);
		onChange({
			...value,
			maxPrice: text.trim() !== "" && amount >= 0 ? amount : null,
		});
	}

	return (
		<div className="flex flex-wrap items-end gap-2">
			<FilterSelect
				id={`${id}-store`}
				label="Store"
				value={value.store}
				choices={STORES}
				onChange={(store) =>
					onChange({ ...value, store: store as ProductFilters["store"] })
				}
			/>
			<FilterSelect
				id={`${id}-measure`}
				label="Size"
				value={value.measure}
				choices={MEASURES}
				onChange={(measure) =>
					onChange({ ...value, measure: measure as ProductFilters["measure"] })
				}
			/>
			<div className="flex flex-col gap-1">
				<label htmlFor={`${id}-price`} className="text-xs">
					Max price ($)
				</label>
				<Input
					id={`${id}-price`}
					type="number"
					inputMode="decimal"
					min={0}
					step="0.01"
					className="w-28"
					value={price}
					onChange={(event) => changePrice(event.target.value)}
				/>
			</div>
		</div>
	);
}

/** One labelled drop-down of the filter bar. */
function FilterSelect({
	id,
	label,
	value,
	choices,
	onChange,
}: {
	id: string;
	label: string;
	value: string;
	choices: ReadonlyArray<{ value: string; label: string }>;
	onChange: (value: string) => void;
}) {
	return (
		<div className="flex flex-col gap-1">
			<label htmlFor={id} className="text-xs">
				{label}
			</label>
			<Select value={value} onValueChange={onChange}>
				<SelectTrigger id={id} className="w-36">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{choices.map((choice) => (
						<SelectItem key={choice.value} value={choice.value}>
							{choice.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	);
}
