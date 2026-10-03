import { createContext, type ReactNode, useContext } from "react";

import { StoreOrderCard } from "#/components/order/store-order-card";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import {
	isEmptyOrder,
	type OrderReview as OrderReviewData,
	type StoreId,
} from "#/lib/api";
import { formatMoney } from "#/lib/money";

const Context = createContext<OrderReviewData | null>(null);

function useReview(): OrderReviewData {
	const review = useContext(Context);
	if (!review)
		throw new Error("OrderReview parts must be inside <OrderReview>");
	return review;
}

/**
 * The order review, as a compound component. The page picks the parts and
 * their order; each part renders only when it has something to show.
 *
 * ```tsx
 * <OrderReview review={review}>
 *   <OrderReview.Empty><Link to="/">Go to the list</Link></OrderReview.Empty>
 *   <OrderReview.Unchosen><Link to="/">Choose products</Link></OrderReview.Unchosen>
 *   <OrderReview.Store store="woolworths"><SendToStore store="woolworths" … /></OrderReview.Store>
 *   <OrderReview.Store store="coles"><SendToStore store="coles" … /></OrderReview.Store>
 *   <OrderReview.Total />
 * </OrderReview>
 * ```
 *
 * Links are passed in as children so these parts never need the router.
 */
function OrderReviewRoot({
	review,
	children,
}: {
	review: OrderReviewData;
	children: ReactNode;
}) {
	return (
		<Context.Provider value={review}>
			<div className="flex flex-col gap-4">{children}</div>
		</Context.Provider>
	);
}

/** Shown only when nothing is committed; `children` lead back to the list. */
function Empty({ children }: { children: ReactNode }) {
	const review = useReview();
	if (!isEmptyOrder(review)) return null;
	return (
		<div className="flex flex-col items-center gap-3 rounded-md border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
			<p>
				Nothing is ready to order yet. On the grocery list, press “Ready to
				order”.
			</p>
			{children}
		</div>
	);
}

/** Committed items with no product chosen; `children` lead to choosing. */
function Unchosen({ children }: { children: ReactNode }) {
	const { unchosen } = useReview();
	if (unchosen.length === 0) return null;
	return (
		<Card role="region" aria-label="Items with no product" className="gap-2">
			<CardHeader>
				<CardTitle>No product chosen yet</CardTitle>
				<CardDescription>
					These items are not in any store's total.
				</CardDescription>
			</CardHeader>
			<CardContent className="flex flex-col gap-3">
				<ul className="flex list-none flex-col gap-1 p-0 text-sm">
					{unchosen.map((item) => (
						<li key={item.grocery_item_id}>
							{item.name}{" "}
							<span className="text-muted-foreground">× {item.quantity}</span>
						</li>
					))}
				</ul>
				{children}
			</CardContent>
		</Card>
	);
}

/** One store's part of the order, if anything is bought there. */
function Store({ store, children }: { store: StoreId; children?: ReactNode }) {
	const order = useReview().stores.find((s) => s.store === store);
	if (!order) return null;
	return <StoreOrderCard order={order}>{children}</StoreOrderCard>;
}

/** The items' total across every store. Delivery is not included. */
function Total() {
	const review = useReview();
	if (isEmptyOrder(review)) return null;
	return (
		<section
			aria-label="Order total"
			className="flex flex-col gap-1 rounded-md border border-border p-4"
		>
			<p className="flex justify-between text-base font-semibold">
				<span>{review.complete ? "Total for items" : "Total so far"}</span>
				<span>{formatMoney(review.total)}</span>
			</p>
			<p className="text-xs text-muted-foreground">
				Delivery fees are not included. You pick a delivery time when you send
				the order to the store.
			</p>
		</section>
	);
}

export const OrderReview = Object.assign(OrderReviewRoot, {
	Empty,
	Unchosen,
	Store,
	Total,
});
