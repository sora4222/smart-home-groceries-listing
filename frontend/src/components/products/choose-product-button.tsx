import { Check } from "lucide-react";
import { useState } from "react";

import {
	isChosen,
	useProductChoice,
} from "#/components/products/product-choice";
import { Button } from "#/components/ui/button";
import type { StoreId, StoreProduct } from "#/lib/api";

/**
 * Chooses one product as the item's product, or shows that it already is.
 *
 * Renders nothing outside a comparison that can choose. An unavailable product
 * cannot be chosen, and the button is disabled while a choice is being saved
 * so a double click does not send two.
 */
export function ChooseProductButton({
	store,
	storeName,
	product,
}: {
	store: StoreId;
	storeName: string;
	product: StoreProduct;
}) {
	const choice = useProductChoice();
	const [saving, setSaving] = useState(false);
	if (!choice) return null;

	const title = [product.brand, product.name].filter(Boolean).join(" ");
	const thisProduct = { store, product_id: product.product_id };
	const chosen = isChosen(choice.chosen, thisProduct);

	async function choose() {
		if (!choice || chosen) return;
		setSaving(true);
		try {
			await choice.onChoose(thisProduct);
		} catch {
			// The page has already said why; the button only has to recover.
		} finally {
			setSaving(false);
		}
	}

	return (
		<Button
			size="sm"
			variant={chosen ? "secondary" : "outline"}
			aria-pressed={chosen}
			aria-label={
				chosen
					? `${title} at ${storeName} is chosen`
					: `Choose ${title} at ${storeName}`
			}
			disabled={saving || !product.available}
			onClick={choose}
		>
			{chosen && <Check aria-hidden />}
			{chosen ? "Chosen" : saving ? "Choosing…" : "Choose"}
		</Button>
	);
}
