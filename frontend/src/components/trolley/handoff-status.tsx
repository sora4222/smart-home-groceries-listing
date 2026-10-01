import { Badge } from "#/components/ui/badge";
import { isExpired } from "#/hooks/useTrolleyHandoff";
import type { TrolleyHandoff } from "#/lib/api";

/** One short sentence for where the handoff is. */
export function handoffSummary(handoff: TrolleyHandoff): string {
	if (isExpired(handoff)) return "Too late — press “Send to Woolworths” again.";
	switch (handoff.status) {
		case "waiting_for_store_tab":
			return "Waiting. On Woolworths, press the “Fill Woolworths trolley” bookmark.";
		case "claimed_by_store_tab":
			return "Adding to your trolley…";
		case "filled":
			return "Done. Check your trolley on Woolworths, then pay there.";
		case "filled_with_problems":
			return "Done, but some products could not be added.";
		case "replaced":
			return "A newer send replaced this one.";
	}
}

/** Where the handoff is, and what happened to each product. */
export function HandoffStatus({ handoff }: { handoff: TrolleyHandoff }) {
	return (
		<section aria-label="Trolley progress" className="flex flex-col gap-2">
			<output className="text-sm font-medium">{handoffSummary(handoff)}</output>
			<ul className="flex flex-col gap-1">
				{handoff.lines.map((line) => (
					<li
						key={line.grocery_item_id}
						className="flex items-center justify-between gap-2 text-sm"
					>
						<span>
							{line.quantity} × {line.name}
							{line.problem && (
								<span className="block text-xs text-muted-foreground">
									{line.problem}
								</span>
							)}
						</span>
						{line.outcome && (
							<Badge
								variant={line.outcome === "added" ? "secondary" : "destructive"}
							>
								{line.outcome === "added" ? "Added" : "Not added"}
							</Badge>
						)}
					</li>
				))}
			</ul>
		</section>
	);
}
