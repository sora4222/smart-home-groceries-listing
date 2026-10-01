import { PriceChangeBadge } from "#/components/order/price-change-badge";
import { Badge } from "#/components/ui/badge";
import type { OrderLine } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * One item of the order: the list item and its quantity, the product chosen
 * for it (linked to the store's page), and what it costs today.
 *
 * A line that cannot be bought as it is shows "No price" and the reason, so
 * the household knows to choose again on the list.
 */
export function OrderLineRow({ line }: { line: OrderLine }) {
	const product = [line.brand, line.product_name].filter(Boolean).join(" ");

	return (
		<li
			aria-label={line.item_name}
			data-item-name={line.item_name}
			className="flex flex-col gap-1 border-b border-border py-2 last:border-b-0"
		>
			<div className="flex items-start justify-between gap-3">
				<div className="flex min-w-0 flex-col">
					<span className="text-sm font-medium">
						{line.item_name}{" "}
						<span className="text-muted-foreground">× {line.quantity}</span>
					</span>
					<span className="text-xs text-muted-foreground">
						<a
							href={line.url}
							target="_blank"
							rel="noreferrer"
							className="underline-offset-4 hover:underline"
						>
							{product}
						</a>
						{line.package_size && ` · ${line.package_size}`}
					</span>
				</div>
				<div className="flex shrink-0 flex-col items-end">
					<span className="text-sm font-medium">
						{line.total_price ? formatMoney(line.total_price) : "No price"}
					</span>
					{line.price && (
						<span className="text-xs text-muted-foreground">
							{formatMoney(line.price)} each
						</span>
					)}
				</div>
			</div>
			<div className="flex flex-wrap gap-1">
				<PriceChangeBadge line={line} />
				{line.deal_applied && <Badge variant="secondary">Deal applied</Badge>}
			</div>
			{line.problem && (
				<p className="text-xs text-destructive">{line.problem}</p>
			)}
		</li>
	);
}
