import { formatMoney } from "#/lib/money";

/** One row of a bar list. */
export interface SpendBar {
	key: string;
	label: string;
	/** Decimal string. */
	amount: string;
	/** A second line under the label, e.g. "2 times · 3 bought". */
	detail?: string;
}

/**
 * Spend as labelled horizontal bars, biggest share longest. The amount is
 * written beside every bar, so nothing is read from the bar's length alone.
 * `renderExtra` adds something under a row (the item drill-down).
 */
export function SpendBars({
	label,
	bars,
	empty,
	renderExtra,
}: {
	label: string;
	bars: SpendBar[];
	empty: string;
	renderExtra?: (bar: SpendBar) => React.ReactNode;
}) {
	if (bars.length === 0) {
		return <p className="text-sm text-muted-foreground">{empty}</p>;
	}
	const largest = Math.max(...bars.map((bar) => Number(bar.amount)), 0);
	return (
		<ul aria-label={label} className="flex list-none flex-col gap-3 p-0">
			{bars.map((bar) => {
				const share = largest > 0 ? (Number(bar.amount) / largest) * 100 : 0;
				return (
					<li key={bar.key} className="flex flex-col gap-1">
						<div className="flex items-baseline justify-between gap-2 text-sm">
							<span className="min-w-0">
								<span className="font-medium">{bar.label}</span>
								{bar.detail && (
									<span className="block text-xs text-muted-foreground">
										{bar.detail}
									</span>
								)}
							</span>
							<span className="shrink-0 tabular-nums">
								{formatMoney(bar.amount)}
							</span>
						</div>
						<div className="h-2 w-full rounded-full bg-muted" aria-hidden>
							<div
								className="h-2 rounded-full bg-primary"
								style={{ width: `${share}%` }}
							/>
						</div>
						{renderExtra?.(bar)}
					</li>
				);
			})}
		</ul>
	);
}
