import { useEffect, useState } from "react";

import { api, type ItemPrice } from "#/lib/api";
import { dayLabel } from "#/lib/day-label";
import { formatMoney } from "#/lib/money";

type State =
	| { status: "loading" }
	| { status: "loaded"; prices: ItemPrice[] }
	| { status: "failed" };

/**
 * What one item cost each time it was bought, at any store, oldest first —
 * to spot a price going up or down. Read when it is opened.
 */
export function ItemPriceHistory({ itemName }: { itemName: string }) {
	const [state, setState] = useState<State>({ status: "loading" });

	useEffect(() => {
		let current = true;
		setState({ status: "loading" });
		api.spending
			.itemPrices(itemName)
			.then((prices) => current && setState({ status: "loaded", prices }))
			.catch(() => current && setState({ status: "failed" }));
		return () => {
			current = false;
		};
	}, [itemName]);

	if (state.status === "loading") {
		return (
			<output className="text-xs text-muted-foreground">Reading prices…</output>
		);
	}
	if (state.status === "failed") {
		return (
			<p role="alert" className="text-xs">
				Could not read the prices just now.
			</p>
		);
	}
	return (
		<table className="w-full text-xs">
			<caption className="sr-only">
				Price paid for {itemName}, oldest first
			</caption>
			<thead className="text-left text-muted-foreground">
				<tr>
					<th className="font-normal">Day</th>
					<th className="font-normal">Product</th>
					<th className="text-right font-normal">Each</th>
				</tr>
			</thead>
			<tbody>
				{state.prices.map((price) => (
					<tr key={`${price.bought_at}-${price.store}-${price.product_name}`}>
						<td className="py-0.5 pr-2 whitespace-nowrap">
							{dayLabel(price.bought_at)}
						</td>
						<td className="py-0.5 pr-2">
							{price.store_name} · {price.product_name}
						</td>
						<td className="py-0.5 text-right tabular-nums">
							{formatMoney(price.unit_price)} × {price.quantity}
						</td>
					</tr>
				))}
			</tbody>
		</table>
	);
}
