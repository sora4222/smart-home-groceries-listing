import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import type { PurchaseOrder } from "#/lib/api";
import { dayLabel } from "#/lib/day-label";
import { formatMoney } from "#/lib/money";

/**
 * The top of the Analysis page: what the newest shop cost, whatever the
 * filters below say.
 */
export function LatestOrderCard({ order }: { order: PurchaseOrder | null }) {
	return (
		<Card role="region" aria-label="Latest shop" className="gap-1">
			<CardHeader>
				<CardDescription>Latest shop</CardDescription>
				<CardTitle className="text-2xl">
					{order ? formatMoney(order.total) : "Nothing yet"}
				</CardTitle>
			</CardHeader>
			<CardContent className="text-sm text-muted-foreground">
				{order ? (
					<>
						{order.store_name} · {dayLabel(order.bought_at)} · items{" "}
						{formatMoney(order.items_total)} + delivery{" "}
						{formatMoney(order.delivery_fee)}
					</>
				) : (
					<>
						A shop shows here after the “Fill Woolworths trolley” bookmark fills
						your trolley.
					</>
				)}
			</CardContent>
		</Card>
	);
}
