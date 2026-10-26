import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";

import { ProductChoiceProvider } from "#/components/products/product-choice";
import { ProductResults } from "#/components/products/product-results";
import { SelectItemPage } from "#/components/products/select-item-page";
import { ChosenProduct } from "#/components/selections/chosen-product";
import { Button } from "#/components/ui/button";
import { useProductChoices } from "#/hooks/useProductChoices";
import { api, selectionsByItem } from "#/lib/api";

export const Route = createFileRoute("/items/$itemId/select")({
	loader: async ({ params }) => {
		const [items, selections] = await Promise.all([
			api.grocery.list(),
			api.selections.list(),
		]);
		const item = items.find((i) => i.id === params.itemId);
		if (!item) throw notFound();
		return { item, selections: selectionsByItem(selections) };
	},
	notFoundComponent: () => (
		<div className="flex flex-col items-start gap-3 text-sm">
			<p>That item is no longer on the list.</p>
			<BackToList />
		</div>
	),
	component: SelectItemRoute,
});

/**
 * `/items/$itemId/select` — choosing the product to buy for one list item.
 * Reached from the item's "Select item" button, not from the top bar.
 */
function SelectItemRoute() {
	const { item, selections } = Route.useLoaderData();
	const choices = useProductChoices(selections);
	const selection = selections.get(item.id);
	const chosen = selection
		? [
				selection,
				...(selection.bought_with ?? []),
				...(selection.also_chosen ?? []),
			]
		: [];

	return (
		<SelectItemPage item={item} back={<BackToList />}>
			<ChosenProduct
				itemName={item.name}
				selection={selection ?? null}
				onClear={(store, productId) => choices.clear(item.id, store, productId)}
				onSetCount={(store, productId, count) =>
					choices.setCount(item.id, store, productId, count)
				}
			/>
			<ProductChoiceProvider
				value={{
					chosen,
					onChoose: (choice) => choices.choose(item.id, choice),
					onUnchoose: (choice) =>
						choices.clear(item.id, choice.store, choice.product_id),
				}}
			>
				<ProductResults item={item} />
			</ProductChoiceProvider>
		</SelectItemPage>
	);
}

/** "Back to the list". */
function BackToList() {
	return (
		<Button asChild variant="ghost" size="sm" className="self-start">
			<Link to="/">
				<ArrowLeft aria-hidden />
				Back to the list
			</Link>
		</Button>
	);
}
