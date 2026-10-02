import { type FormEvent, useEffect, useId, useState } from "react";

import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import type { AnalysisSearch } from "#/lib/analysis-search";
import type { StoreId } from "#/lib/api";
import { RANGE_PRESETS, type RangePreset } from "#/lib/date-range";

const SELECT_CLASS =
	"h-9 rounded-md border border-input bg-background px-2 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none";

/**
 * The filters every view shares, in one row above the views: date range,
 * store, item name and category. Each change updates the page address,
 * which re-reads the spending.
 */
export function SpendingFilters({
	search,
	categories,
	onChange,
}: {
	search: AnalysisSearch;
	categories: string[];
	onChange: (next: AnalysisSearch) => void;
}) {
	const range = search.range ?? "all";
	const fromId = useId();
	const toId = useId();
	const datesId = useId();
	const storeId = useId();
	const categoryId = useId();
	const set = (patch: Partial<AnalysisSearch>) =>
		onChange({ ...search, ...patch });

	return (
		<fieldset className="flex flex-wrap items-end gap-3">
			<legend className="sr-only">Filters</legend>
			<div className="flex flex-col gap-1">
				<label htmlFor={datesId} className="text-xs text-muted-foreground">
					Dates
				</label>
				<select
					id={datesId}
					className={SELECT_CLASS}
					value={range}
					onChange={(e) => set({ range: e.target.value as RangePreset })}
				>
					{RANGE_PRESETS.map((preset) => (
						<option key={preset.value} value={preset.value}>
							{preset.label}
						</option>
					))}
				</select>
			</div>
			{range === "custom" && (
				<>
					<label
						htmlFor={fromId}
						className="flex flex-col gap-1 text-xs text-muted-foreground"
					>
						From
						<Input
							id={fromId}
							type="date"
							className="w-40"
							value={search.from ?? ""}
							onChange={(e) => set({ from: e.target.value || undefined })}
						/>
					</label>
					<label
						htmlFor={toId}
						className="flex flex-col gap-1 text-xs text-muted-foreground"
					>
						To
						<Input
							id={toId}
							type="date"
							className="w-40"
							value={search.to ?? ""}
							onChange={(e) => set({ to: e.target.value || undefined })}
						/>
					</label>
				</>
			)}
			<div className="flex flex-col gap-1">
				<label htmlFor={storeId} className="text-xs text-muted-foreground">
					Store
				</label>
				<select
					id={storeId}
					className={SELECT_CLASS}
					value={search.store ?? ""}
					onChange={(e) =>
						set({ store: (e.target.value || undefined) as StoreId | undefined })
					}
				>
					<option value="">Both stores</option>
					<option value="woolworths">Woolworths</option>
					<option value="coles">Coles</option>
				</select>
			</div>
			<div className="flex flex-col gap-1">
				<label htmlFor={categoryId} className="text-xs text-muted-foreground">
					Category
				</label>
				<select
					id={categoryId}
					className={SELECT_CLASS}
					value={search.category ?? ""}
					onChange={(e) => set({ category: e.target.value || undefined })}
				>
					<option value="">All categories</option>
					{categories.map((category) => (
						<option key={category} value={category}>
							{category}
						</option>
					))}
				</select>
			</div>
			<ItemSearch
				value={search.item ?? ""}
				onSearch={(item) => set({ item: item || undefined })}
			/>
		</fieldset>
	);
}

/** Item name search, applied on Enter or the Find button. */
function ItemSearch({
	value,
	onSearch,
}: {
	value: string;
	onSearch: (item: string) => void;
}) {
	const [text, setText] = useState(value);
	const id = useId();
	useEffect(() => setText(value), [value]);

	function submit(event: FormEvent) {
		event.preventDefault();
		onSearch(text.trim());
	}

	return (
		<form onSubmit={submit} className="flex items-end gap-2">
			<label
				htmlFor={id}
				className="flex flex-col gap-1 text-xs text-muted-foreground"
			>
				Item name
				<Input
					id={id}
					type="search"
					className="w-40"
					value={text}
					onChange={(e) => setText(e.target.value)}
				/>
			</label>
			<Button type="submit" variant="outline" size="sm" className="h-9">
				Find
			</Button>
		</form>
	);
}
