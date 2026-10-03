import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import type { ProductChoice, ProductDislike } from "#/lib/api";
import { groupByMember } from "#/lib/dislikes";

/**
 * The household view of dislikes: one card per member, the caller first.
 *
 * Only the caller's own dislikes have a **Remove** button — nobody removes
 * another member's dislike. Removing is the spec's permanent override.
 */
export function HouseholdDislikes({
	dislikes,
	onRemove,
}: {
	dislikes: ProductDislike[];
	onRemove: (choice: ProductChoice) => void;
}) {
	const groups = groupByMember(dislikes);
	if (groups.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				Nobody dislikes a product yet. Press Dislike on a product when you
				compare prices.
			</p>
		);
	}

	return (
		<div className="flex flex-col gap-4">
			{groups.map((group) => (
				<Card key={group.userId} className="gap-0 py-0">
					<section
						aria-label={
							group.mine ? "Your dislikes" : `${group.userName}'s dislikes`
						}
					>
						<h2 className="px-4 pt-4 text-sm font-semibold">
							{group.mine ? "Your dislikes" : `${group.userName}'s dislikes`}
						</h2>
						<ul className="divide-y divide-border px-4">
							{group.dislikes.map((d) => (
								<DislikeLine
									key={`${d.store}-${d.product_id}`}
									dislike={d}
									onRemove={onRemove}
								/>
							))}
						</ul>
					</section>
				</Card>
			))}
		</div>
	);
}

/** One disliked product, with Remove when it is the caller's own. */
function DislikeLine({
	dislike,
	onRemove,
}: {
	dislike: ProductDislike;
	onRemove: (choice: ProductChoice) => void;
}) {
	const title = [dislike.brand, dislike.name].filter(Boolean).join(" ");
	return (
		<li
			data-testid="household-dislike"
			className="flex flex-wrap items-center justify-between gap-2 py-3"
		>
			<div className="flex min-w-0 flex-col">
				<span className="font-medium">{title}</span>
				<span className="text-xs text-muted-foreground">
					{[dislike.package_size, dislike.store_name]
						.filter(Boolean)
						.join(" · ")}
				</span>
			</div>
			{dislike.mine && (
				<Button
					size="sm"
					variant="outline"
					aria-label={`Remove my dislike of ${title} at ${dislike.store_name}`}
					onClick={() =>
						onRemove({ store: dislike.store, product_id: dislike.product_id })
					}
				>
					Remove
				</Button>
			)}
		</li>
	);
}
