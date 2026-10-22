import type { StoreProducts } from "#/lib/api";
import { visibleProducts } from "#/lib/specials";

/**
 * Why a store shows no products in the comparison: it could not be
 * searched, nothing matched the item's filters, or nothing is on special.
 * Stores with products to show say nothing.
 */
export function StoreNotices({
	stores,
	specialsOnly,
}: {
	stores: StoreProducts[];
	specialsOnly: boolean;
}) {
	const notices = stores.flatMap((results) => {
		const message = noticeFor(results, specialsOnly);
		return message ? [{ store: results.store, message }] : [];
	});
	if (notices.length === 0) return null;

	return (
		<div className="flex flex-col gap-1">
			{notices.map(({ store, message }) => (
				<output
					key={store}
					data-store={store}
					className="block text-sm text-muted-foreground"
				>
					{message}
				</output>
			))}
		</div>
	);
}

/** The sentence for one store, or `null` when it has products to show. */
function noticeFor(
	results: StoreProducts,
	specialsOnly: boolean,
): string | null {
	if (results.status !== "ok") {
		return results.message ?? `${results.store_name} could not be searched.`;
	}
	if (visibleProducts(results.products, specialsOnly).length > 0) return null;
	return results.products.length > 0
		? `Nothing on special at ${results.store_name}.`
		: `No products at ${results.store_name} match this item and its filters.`;
}
