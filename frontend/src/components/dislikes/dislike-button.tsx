import { ThumbsDown } from "lucide-react";
import { useState } from "react";

import { useProductDislikes } from "#/components/dislikes/product-dislikes";
import { Button } from "#/components/ui/button";
import type { StoreId, StoreProduct } from "#/lib/api";
import { dislikesOf } from "#/lib/dislikes";

/**
 * The caller's own dislike of one product: "Dislike", or "Remove my dislike"
 * when they already dislike it. Removing is the spec's permanent override.
 *
 * Renders nothing outside a price comparison. Disabled while saving, so a
 * double click sends one request.
 */
export function DislikeButton({
	store,
	storeName,
	product,
}: {
	store: StoreId;
	storeName: string;
	product: StoreProduct;
}) {
	const context = useProductDislikes();
	const [saving, setSaving] = useState(false);
	if (!context) return null;

	const choice = { store, product_id: product.product_id };
	const mine = dislikesOf(context.dislikes, choice).some((d) => d.mine);
	const title = [product.brand, product.name].filter(Boolean).join(" ");

	async function toggle() {
		if (!context) return;
		setSaving(true);
		try {
			if (mine) {
				await context.removeMine(choice);
			} else {
				await context.dislike({
					...choice,
					name: product.name,
					brand: product.brand,
					package_size: product.package_size,
				});
			}
		} finally {
			setSaving(false);
		}
	}

	return (
		<Button
			size="sm"
			variant="ghost"
			aria-label={
				mine
					? `Remove my dislike of ${title} at ${storeName}`
					: `Dislike ${title} at ${storeName}`
			}
			disabled={saving}
			onClick={toggle}
		>
			<ThumbsDown aria-hidden />
			{mine ? "Remove my dislike" : "Dislike"}
		</Button>
	);
}
