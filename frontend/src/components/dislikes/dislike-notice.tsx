import { TriangleAlert } from "lucide-react";
import { useState } from "react";

import { useProductDislikes } from "#/components/dislikes/product-dislikes";
import { Button } from "#/components/ui/button";
import type { StoreId, StoreProduct } from "#/lib/api";
import { dislikesOf, dislikeWarning, isOverridden } from "#/lib/dislikes";

/**
 * The warning on a disliked product: "Phu disliked this item previously."
 *
 * The product can still be chosen. "Buy it this time" sets the dislike
 * aside for this item only (the spec's per-order override); "Undo" brings
 * it back. Renders nothing when nobody dislikes the product, or outside a
 * price comparison.
 */
export function DislikeNotice({
	store,
	product,
}: {
	store: StoreId;
	product: StoreProduct;
}) {
	const context = useProductDislikes();
	const [saving, setSaving] = useState(false);
	if (!context) return null;

	const choice = { store, product_id: product.product_id };
	const dislikes = dislikesOf(context.dislikes, choice);
	if (dislikes.length === 0) return null;
	const overridden = isOverridden(context.overrides, context.itemId, choice);

	async function toggle() {
		if (!context) return;
		setSaving(true);
		try {
			await (overridden
				? context.clearOverride(choice)
				: context.override(choice));
		} finally {
			setSaving(false);
		}
	}

	return (
		<div
			data-testid="dislike-notice"
			className="flex flex-wrap items-center gap-2 rounded-md border border-border bg-muted px-2 py-1 text-xs"
		>
			<TriangleAlert aria-hidden className="size-3 text-destructive" />
			<output>{dislikeWarning(dislikes)}</output>
			{overridden && (
				<span className="text-muted-foreground">OK to buy it this time.</span>
			)}
			<Button
				size="sm"
				variant="outline"
				className="ml-auto"
				disabled={saving}
				onClick={toggle}
			>
				{overridden ? "Undo" : "Buy it this time"}
			</Button>
		</div>
	);
}
