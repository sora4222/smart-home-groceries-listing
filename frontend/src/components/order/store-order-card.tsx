import type { ReactNode } from "react";

import { OrderLineRow } from "#/components/order/order-line-row";
import {
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import type { StoreOrder } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * Everything bought at one store: its lines and subtotal. `children` are the
 * store's actions (Send to Woolworths), shown under the subtotal.
 *
 * The subtotal reads "so far" while any line has no price today.
 */
export function StoreOrderCard({
	order,
	children,
}: {
	order: StoreOrder;
	children?: ReactNode;
}) {
	const count = order.lines.length;

	return (
		<Card role="region" aria-label={order.store_name} className="gap-2">
			<CardHeader>
				<CardTitle>{order.store_name}</CardTitle>
				<CardDescription>
					{count} {count === 1 ? "item" : "items"}
				</CardDescription>
			</CardHeader>
			<CardContent>
				<ul className="flex list-none flex-col p-0">
					{order.lines.map((line) => (
						<OrderLineRow key={line.grocery_item_id} line={line} />
					))}
				</ul>
			</CardContent>
			<CardFooter className="flex flex-col items-stretch gap-3">
				<p className="flex justify-between text-sm font-medium">
					<span>{order.complete ? "Subtotal" : "Subtotal so far"}</span>
					<span>{formatMoney(order.subtotal)}</span>
				</p>
				{children}
			</CardFooter>
		</Card>
	);
}
