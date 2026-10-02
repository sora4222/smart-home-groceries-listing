import { ItemSpendList } from "#/components/analysis/item-spend-list";
import { OverTimeChart } from "#/components/analysis/over-time-chart";
import { SpendBars } from "#/components/analysis/spend-bars";
import { Button } from "#/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import type { AnalysisView } from "#/lib/analysis-search";
import type { SpendingPeriod, SpendingReport } from "#/lib/api";
import { formatMoney } from "#/lib/money";

const PERIODS: ReadonlyArray<{ value: SpendingPeriod; label: string }> = [
	{ value: "week", label: "Weeks" },
	{ value: "month", label: "Months" },
	{ value: "quarter", label: "Quarters" },
];

/**
 * The four views of the same spending, switched with tabs: over time, by
 * item, by store and by category.
 */
export function SpendingViews({
	report,
	view,
	period,
	onViewChange,
	onPeriodChange,
	onRecategorise,
}: {
	report: SpendingReport;
	view: AnalysisView;
	period: SpendingPeriod;
	onViewChange: (view: AnalysisView) => void;
	onPeriodChange: (period: SpendingPeriod) => void;
	onRecategorise: () => void;
}) {
	return (
		<Tabs value={view} onValueChange={(v) => onViewChange(v as AnalysisView)}>
			<TabsList className="w-full sm:w-fit">
				<TabsTrigger value="time">Over time</TabsTrigger>
				<TabsTrigger value="item">By item</TabsTrigger>
				<TabsTrigger value="store">By store</TabsTrigger>
				<TabsTrigger value="category">By category</TabsTrigger>
			</TabsList>

			<p className="text-sm">
				Spent:{" "}
				<span className="font-semibold">{formatMoney(report.total)}</span>{" "}
				<span className="text-muted-foreground">
					(items {formatMoney(report.items_total)} + delivery{" "}
					{formatMoney(report.delivery_total)})
				</span>
			</p>

			<TabsContent value="time" className="flex flex-col gap-3">
				<fieldset className="flex gap-1">
					<legend className="sr-only">Group by</legend>
					{PERIODS.map((p) => (
						<Button
							key={p.value}
							size="sm"
							variant={p.value === period ? "secondary" : "ghost"}
							aria-pressed={p.value === period}
							onClick={() => onPeriodChange(p.value)}
						>
							{p.label}
						</Button>
					))}
				</fieldset>
				<OverTimeChart points={report.over_time} period={period} />
			</TabsContent>

			<TabsContent value="item">
				<ItemSpendList items={report.by_item} />
			</TabsContent>

			<TabsContent value="store">
				<SpendBars
					label="Spend by store"
					empty="Nothing bought in this range."
					bars={report.by_store.map((store) => ({
						key: store.store,
						label: store.store_name,
						amount: store.total,
						detail: `items ${formatMoney(store.items_total)} + delivery ${formatMoney(store.delivery_total)}`,
					}))}
				/>
			</TabsContent>

			<TabsContent value="category" className="flex flex-col gap-3">
				<SpendBars
					label="Spend by category"
					empty="Nothing bought in this range."
					bars={report.by_category.map((c) => ({
						key: c.category,
						label: c.category,
						amount: c.total,
					}))}
				/>
				<Button
					size="sm"
					variant="ghost"
					className="self-start text-xs text-muted-foreground"
					onClick={onRecategorise}
				>
					Sort categories again
				</Button>
			</TabsContent>
		</Tabs>
	);
}
