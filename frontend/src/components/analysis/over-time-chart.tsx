import { useState } from "react";

import type { PeriodSpend, SpendingPeriod } from "#/lib/api";
import { formatMoney } from "#/lib/money";
import { periodLabel, periodTick } from "#/lib/period-label";

const WIDTH = 600;
const HEIGHT = 220;
const PAD = { top: 16, right: 16, bottom: 28, left: 56 };

/**
 * "Over time": total spend (delivery included) per week, month or quarter,
 * as a line. Hover or focus a point to read it; the same numbers are in a
 * table under the chart.
 */
export function OverTimeChart({
	points,
	period,
}: {
	points: PeriodSpend[];
	period: SpendingPeriod;
}) {
	const [active, setActive] = useState<number | null>(null);
	if (points.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				Nothing bought in this range.
			</p>
		);
	}

	const values = points.map((p) => Number(p.total));
	const max = Math.max(...values, 0) || 1;
	const plotW = WIDTH - PAD.left - PAD.right;
	const plotH = HEIGHT - PAD.top - PAD.bottom;
	const x = (i: number) =>
		PAD.left +
		(points.length === 1 ? plotW / 2 : (i / (points.length - 1)) * plotW);
	const y = (v: number) => PAD.top + plotH - (v / max) * plotH;
	const line = values
		.map((v, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(v)}`)
		.join(" ");
	const shown = active ?? points.length - 1;
	const ticks = tickIndexes(points.length);

	return (
		<figure className="flex flex-col gap-2">
			<output className="text-sm" aria-live="polite">
				{periodLabel(period, points[shown].start)}:{" "}
				<span className="font-semibold">
					{formatMoney(points[shown].total)}
				</span>
			</output>
			<svg
				viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
				className="h-auto w-full"
				role="img"
				aria-label={`Spend per ${period}, ${points.length} points, highest ${formatMoney(String(max))}`}
			>
				{[0, max].map((v) => (
					<g key={v}>
						<line
							x1={PAD.left}
							x2={WIDTH - PAD.right}
							y1={y(v)}
							y2={y(v)}
							className="stroke-border"
							strokeWidth={1}
						/>
						<text
							x={PAD.left - 8}
							y={y(v) + 4}
							textAnchor="end"
							className="fill-muted-foreground text-xs"
						>
							{formatMoney(String(v))}
						</text>
					</g>
				))}
				{ticks.map((i) => (
					<text
						key={points[i].start}
						x={x(i)}
						y={HEIGHT - 8}
						textAnchor={tickAnchor(i, points.length)}
						className="fill-muted-foreground text-xs"
					>
						{periodTick(period, points[i].start)}
					</text>
				))}
				<path d={line} fill="none" className="stroke-primary" strokeWidth={2} />
				{points.map((p, i) => (
					// biome-ignore lint/a11y/useSemanticElements: an SVG point has no HTML equivalent; it is focusable so keyboards can read each value.
					<g
						key={p.start}
						role="button"
						tabIndex={0}
						aria-label={`${periodLabel(period, p.start)}: ${formatMoney(p.total)}`}
						onMouseEnter={() => setActive(i)}
						onFocus={() => setActive(i)}
						onClick={() => setActive(i)}
						onKeyDown={(event) => event.key === "Enter" && setActive(i)}
						className="cursor-pointer outline-none"
					>
						<circle cx={x(i)} cy={y(values[i])} r={14} fill="transparent" />
						<circle
							cx={x(i)}
							cy={y(values[i])}
							r={shown === i ? 5 : 4}
							className="fill-primary stroke-background"
							strokeWidth={2}
						/>
					</g>
				))}
			</svg>
			<details className="text-sm">
				<summary className="cursor-pointer text-muted-foreground">
					Show as a table
				</summary>
				<table className="mt-2 w-full">
					<thead className="text-left text-muted-foreground">
						<tr>
							<th className="font-normal">When</th>
							<th className="text-right font-normal">Spent</th>
						</tr>
					</thead>
					<tbody>
						{points.map((p) => (
							<tr key={p.start}>
								<td>{periodLabel(period, p.start)}</td>
								<td className="text-right tabular-nums">
									{formatMoney(p.total)}
								</td>
							</tr>
						))}
					</tbody>
				</table>
			</details>
		</figure>
	);
}

/** Axis labels: every point up to six, else five spread out, always the first and last. */
export function tickIndexes(count: number): number[] {
	if (count <= 6) return Array.from({ length: count }, (_, i) => i);
	const step = (count - 1) / 4;
	return [0, 1, 2, 3, 4].map((n) => Math.round(n * step));
}

/** The first and last labels sit inside the chart instead of past its edges. */
function tickAnchor(index: number, count: number): "start" | "middle" | "end" {
	if (count > 1 && index === 0) return "start";
	if (count > 1 && index === count - 1) return "end";
	return "middle";
}
