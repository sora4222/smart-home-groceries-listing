/**
 * Reading dislikes for one product: who dislikes it, whether it is set aside
 * for an item, and the warning sentence people see. Pure, no fetching.
 */
import type { DislikeOverride, ProductChoice, ProductDislike } from "#/lib/api";

/** Every member's dislike of this product at this store, oldest first. */
export function dislikesOf(
	dislikes: ProductDislike[],
	product: ProductChoice,
): ProductDislike[] {
	return dislikes
		.filter(
			(d) => d.store === product.store && d.product_id === product.product_id,
		)
		.sort((a, b) => a.disliked_at.localeCompare(b.disliked_at));
}

/** Whether someone said "buy it this time anyway" for this item. */
export function isOverridden(
	overrides: DislikeOverride[],
	itemId: string,
	product: ProductChoice,
): boolean {
	return overrides.some(
		(o) =>
			o.grocery_item_id === itemId &&
			o.store === product.store &&
			o.product_id === product.product_id,
	);
}

/**
 * The spec's warning: "Phu disliked this item previously." The caller's own
 * dislike reads "You". Several names are joined with commas and "and".
 */
export function dislikeWarning(dislikes: ProductDislike[]): string {
	const names = dislikes.map((d) => (d.mine ? "You" : d.user_name));
	return `${joinNames(names)} disliked this item previously.`;
}

/** "A", "A and B", "A, B and C". */
function joinNames(names: string[]): string {
	if (names.length <= 1) return names[0] ?? "";
	return `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

/** One member's dislikes, for the household view. */
export interface MemberDislikes {
	userId: string;
	userName: string;
	mine: boolean;
	dislikes: ProductDislike[];
}

/**
 * The household view: dislikes grouped by member, the caller first, then
 * the others by name. Each group keeps the order it was given (newest first
 * from the backend).
 */
export function groupByMember(dislikes: ProductDislike[]): MemberDislikes[] {
	const groups = new Map<string, MemberDislikes>();
	for (const d of dislikes) {
		const group = groups.get(d.user_id) ?? {
			userId: d.user_id,
			userName: d.user_name,
			mine: d.mine,
			dislikes: [],
		};
		group.dislikes.push(d);
		groups.set(d.user_id, group);
	}
	return [...groups.values()].sort(
		(a, b) =>
			Number(b.mine) - Number(a.mine) || a.userName.localeCompare(b.userName),
	);
}
