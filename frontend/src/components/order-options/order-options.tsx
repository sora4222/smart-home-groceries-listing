import type { ReactNode } from "react";

import { OptionCard } from "#/components/order-options/option-card";
import type { OrderOption, OrderOptions as OrderOptionsData } from "#/lib/api";

/**
 * The ways to buy the committed list, best first, each with delivery fees.
 *
 * `children` are shown when a delivery fee is not set yet (a link to
 * Settings › Delivery), so this part never needs the router. Renders nothing
 * when there is nothing to buy.
 */
export function OrderOptions({
	plan,
	onUse,
	children,
}: {
	plan: OrderOptionsData;
	onUse: (option: OrderOption) => Promise<void>;
	children?: ReactNode;
}) {
	if (plan.options.length === 0) return null;
	const feesMissing = plan.options.some((option) => !option.fees_known);
	return (
		<section aria-label="Ways to buy" className="flex flex-col gap-3">
			<div className="flex flex-col gap-1">
				<h2 className="text-base font-semibold">Ways to buy</h2>
				<p className="text-xs text-muted-foreground">
					Each total includes delivery. The app only moves items between
					products you chose.
					{!plan.exact &&
						" Many items can go either way, so the mix shown is a very good one, not every combination."}
				</p>
			</div>
			{feesMissing && children}
			{plan.options.map((option) => (
				<OptionCard
					key={option.kind}
					option={option}
					maxDeliverySpend={plan.max_delivery_spend}
					onUse={onUse}
				/>
			))}
		</section>
	);
}
