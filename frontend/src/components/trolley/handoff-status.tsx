import { SavedPurchase } from "#/components/trolley/saved-purchase";
import { Badge } from "#/components/ui/badge";
import { isExpired } from "#/hooks/useTrolleyHandoff";
import type { TrolleyHandoff, TrolleyHandoffDelivery } from "#/lib/api";
import { storeDayLabel } from "#/lib/delivery-days";
import { formatMoney } from "#/lib/money";

/** One sentence about the delivery time, once the store tab has reported. */
export function deliverySummary(
	delivery: TrolleyHandoffDelivery,
): string | null {
	if (delivery.outcome === null) return null;
	if (delivery.outcome === "failed") {
		return `No delivery time reserved: ${delivery.problem ?? "unknown reason"}. Pick one on Woolworths.`;
	}
	const when = delivery.window_start
		? `${storeDayLabel(delivery.window_start)}, ${delivery.window_label}`
		: delivery.window_label;
	const fee = delivery.fee ? ` (${formatMoney(delivery.fee)})` : "";
	const kept =
		delivery.outcome === "kept" ? " — kept the time you already had" : "";
	return `Delivery: ${when}${fee}${kept}. You can change it on Woolworths.`;
}

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

/** Where the handoff is, what happened to each product, and — once filled —
 * the shop saved as bought, with Undo. */
export function HandoffStatus({ handoff }: { handoff: TrolleyHandoff }) {
	return (
		<section aria-label="Trolley progress" className="flex flex-col gap-2">
			<output className="text-sm font-medium">{handoffSummary(handoff)}</output>
			{deliverySummary(handoff.delivery) && (
				<p className="text-sm">{deliverySummary(handoff.delivery)}</p>
			)}
			{handoff.delivery.problem && handoff.delivery.outcome !== "failed" && (
				<p className="text-xs text-muted-foreground">
					{handoff.delivery.problem}
				</p>
			)}
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
			<SavedPurchase handoff={handoff} />
		</section>
	);
}
