import { createFileRoute, Link, useRouter } from "@tanstack/react-router";
import { useState } from "react";

import { OrderReview } from "#/components/order/order-review";
import { SendToStore } from "#/components/trolley/send-to-store";
import { Button } from "#/components/ui/button";
import { api, selectionsByItem } from "#/lib/api";
import { chosenAtStore } from "#/lib/chosen-at-store";

export const Route = createFileRoute("/order")({
	// The order re-priced now, plus the list and choices so the Send button
	// counts exactly what a handoff would send.
	loader: async () => {
		const [review, items, selections] = await Promise.all([
			api.orderReview.get(),
			api.grocery.list(),
			api.selections.list(),
		]);
		const chosen = selectionsByItem(selections);
		return {
			review,
			chosenAtWoolworths: chosenAtStore(items, chosen, "woolworths"),
			chosenAtColes: chosenAtStore(items, chosen, "coles"),
		};
	},
	pendingComponent: () => (
		<p className="text-sm text-muted-foreground">Checking today's prices…</p>
	),
	errorComponent: OrderError,
	component: OrderPage,
});

/**
 * `/order` — the order review.
 *
 * Shows every committed item with the product chosen for it, priced again at
 * its store now, grouped by store with subtotals and a total for the items.
 * Items with no product, and products a store no longer sells, send the
 * household back to the list to choose. Nothing here chooses for them.
 *
 * "Check prices again" re-runs the loader; the backend answers most of it
 * from its 10-minute search cache.
 */
function OrderPage() {
	const { review, chosenAtWoolworths, chosenAtColes } =
		Route.useLoaderData();
	const router = useRouter();
	const [checking, setChecking] = useState(false);

	async function checkAgain() {
		setChecking(true);
		try {
			await router.invalidate({ sync: true });
		} finally {
			setChecking(false);
		}
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-wrap items-center justify-between gap-2">
				<h1 className="text-lg font-semibold">Order</h1>
				<Button
					variant="outline"
					size="sm"
					disabled={checking}
					onClick={checkAgain}
				>
					{checking ? "Checking…" : "Check prices again"}
				</Button>
			</div>

			<OrderReview review={review}>
				<OrderReview.Empty>
					<Button asChild variant="outline">
						<Link to="/">Go to the grocery list</Link>
					</Button>
				</OrderReview.Empty>
				<OrderReview.Unchosen>
					<Button asChild variant="outline" className="w-full sm:w-auto">
						<Link to="/">Choose products on the list</Link>
					</Button>
				</OrderReview.Unchosen>
				<OrderReview.Store store="woolworths">
					<SendToStore store="woolworths" chosenCount={chosenAtWoolworths}>
						<SendToStore.Trigger />
						<SendToStore.Content />
					</SendToStore>
				</OrderReview.Store>
				<OrderReview.Store store="coles">
					<SendToStore store="coles" chosenCount={chosenAtColes}>
						<SendToStore.Trigger />
						<SendToStore.Content />
					</SendToStore>
				</OrderReview.Store>
				<OrderReview.Total />
			</OrderReview>
		</div>
	);
}

/** The order could not be read; offer to try again. */
function OrderError() {
	const router = useRouter();
	return (
		<div role="alert" className="flex flex-col items-start gap-3 text-sm">
			<p>Could not check the order just now.</p>
			<Button variant="outline" onClick={() => router.invalidate()}>
				Try again
			</Button>
		</div>
	);
}
