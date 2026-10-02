/**
 * Purchase history: `/api/purchase-orders` and `/api/purchase-history/*`.
 *
 * A shop is saved as bought by the backend when the "Fill trolley" bookmark
 * reports that it filled the store's trolley. The web app reads the saved
 * shops, undoes one saved by mistake, and asks how often a product was
 * bought for the product picker. Money is a decimal string.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";

/** A saved shop at one store. */
export interface PurchaseOrder {
	id: string;
	store: StoreId;
	store_name: string;
	source: "trolley_fill";
	/** The trolley fill it was saved from. */
	trolley_handoff_id: string | null;
	bought_at: string;
	items_total: string;
	delivery_fee: string;
	/** Items plus delivery. */
	total: string;
	/** How many products it holds; `null` where not counted. */
	products: number | null;
}

/** One past purchase of a product. */
export interface PastPurchase {
	bought_at: string;
	quantity: number;
	/** What one cost, deals applied. */
	unit_price: string;
	total_price: string;
}

/** One product's past purchases, newest first. */
export interface ProductHistory {
	store: StoreId;
	product_id: string;
	times_bought: number;
	purchases: PastPurchase[];
}

/** A product at a store, as a lookup names it. */
export interface ProductRef {
	store: StoreId;
	product_id: string;
}

export const purchasesApi = {
	/** The newest saved shops, newest first. */
	listOrders: () => request<PurchaseOrder[]>("/api/purchase-orders"),
	/** Undo: forgets a saved shop and puts its items back on the list. */
	undo: (orderId: string) =>
		request<{ items_restored: number }>(`/api/purchase-orders/${orderId}`, {
			method: "DELETE",
		}),
	/** Past purchases of these products. Products never bought are left out. */
	productHistory: (products: ProductRef[]) =>
		request<ProductHistory[]>("/api/purchase-history/products", {
			method: "POST",
			body: JSON.stringify({ products }),
		}),
	/** Works out every purchase's category again. */
	recategorise: () =>
		request<{ changed: number }>("/api/purchase-history/recategorise", {
			method: "POST",
		}),
};

/** The key a product's history is found under. */
export function productKey(store: StoreId, productId: string): string {
	return `${store}:${productId}`;
}
