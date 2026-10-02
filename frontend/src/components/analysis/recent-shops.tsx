import { Button } from "#/components/ui/button";
import { useUndoPurchase } from "#/hooks/useUndoPurchase";
import type { PurchaseOrder } from "#/lib/api";
import { dayLabel } from "#/lib/day-label";
import { formatMoney } from "#/lib/money";

/**
 * The newest saved shops, each with Undo for one saved by mistake. Undo
 * puts the shop's items back on the list; `onChanged` re-reads the page.
 */
export function RecentShops({
	orders,
	onChanged,
}: {
	orders: PurchaseOrder[];
	onChanged: () => void;
}) {
	const { busyId, undo } = useUndoPurchase();
	if (orders.length === 0) return null;

	return (
		<section aria-labelledby="recent-shops" className="flex flex-col gap-2">
			<h2 id="recent-shops" className="text-sm font-semibold">
				Saved shops
			</h2>
			<ul className="flex list-none flex-col divide-y divide-border p-0">
				{orders.map((order) => (
					<li
						key={order.id}
						className="flex items-center justify-between gap-2 py-2 text-sm"
					>
						<span>
							{dayLabel(order.bought_at)} · {order.store_name}
							<span className="block text-xs text-muted-foreground">
								{order.products ?? 0}{" "}
								{order.products === 1 ? "product" : "products"} ·{" "}
								{formatMoney(order.total)}
							</span>
						</span>
						<Button
							size="sm"
							variant="outline"
							disabled={busyId === order.id}
							aria-label={`Undo the ${order.store_name} shop on ${dayLabel(order.bought_at)}`}
							onClick={async () => {
								if (await undo(order)) onChanged();
							}}
						>
							{busyId === order.id ? "Undoing…" : "Undo"}
						</Button>
					</li>
				))}
			</ul>
		</section>
	);
}
