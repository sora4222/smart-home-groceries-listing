/**
 * Product dislikes, per household member: `/api/product-dislikes`, and
 * "buy it this time anyway" for one list item: `/api/dislike-overrides` and
 * `/api/grocery-items/{id}/dislike-override`.
 *
 * A dislike never hides a product. It only puts a warning on it, and the
 * automated order (not built yet) skips it.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";
import type { ProductChoice } from "#/lib/api/selections";

/** One member's dislike of one store product. */
export interface ProductDislike {
	store: StoreId;
	store_name: string;
	product_id: string;
	name: string;
	brand: string | null;
	package_size: string | null;
	user_id: string;
	/** What the warning calls the member. */
	user_name: string;
	/** The caller's own dislike, so the caller may remove it. */
	mine: boolean;
	disliked_at: string;
}

/** What is sent to dislike a product: the label the comparison showed. */
export interface DislikedProduct {
	store: StoreId;
	product_id: string;
	name: string;
	brand: string | null;
	package_size: string | null;
}

/** A dislike set aside for one list item ("buy it this time anyway"). */
export interface DislikeOverride {
	grocery_item_id: string;
	store: StoreId;
	product_id: string;
	overridden_by: string;
	overridden_at: string;
}

const overridePath = (itemId: string) =>
	`/api/grocery-items/${itemId}/dislike-override`;

export const dislikesApi = {
	/** Every member's dislikes, newest first. */
	list: () => request<ProductDislike[]>("/api/product-dislikes"),
	/** The caller dislikes the product. Doing it twice is fine. */
	dislike: (product: DislikedProduct) =>
		request<ProductDislike>("/api/product-dislikes", {
			method: "PUT",
			body: JSON.stringify(product),
		}),
	/** Removes the caller's own dislike, for good. */
	removeMine: (choice: ProductChoice) =>
		request<void>(
			`/api/product-dislikes/${choice.store}/${encodeURIComponent(choice.product_id)}`,
			{ method: "DELETE" },
		),
	/** Every "buy it this time anyway" on the list. */
	overrides: () => request<DislikeOverride[]>("/api/dislike-overrides"),
	/** Buy the disliked product for this item anyway. */
	override: (itemId: string, choice: ProductChoice) =>
		request<DislikeOverride>(overridePath(itemId), {
			method: "PUT",
			body: JSON.stringify(choice),
		}),
	/** The dislikes count again for this item. */
	clearOverride: (itemId: string, choice: ProductChoice) =>
		request<void>(
			`${overridePath(itemId)}/${choice.store}/${encodeURIComponent(choice.product_id)}`,
			{ method: "DELETE" },
		),
};
