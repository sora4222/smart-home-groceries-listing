import { useState } from "react";

import { Button } from "#/components/ui/button";
import { useSavedPurchase } from "#/hooks/useSavedPurchase";
import { useUndoPurchase } from "#/hooks/useUndoPurchase";
import type { TrolleyHandoff } from "#/lib/api";
import { formatMoney } from "#/lib/money";

/**
 * After the bookmark filled the trolley: "Saved as bought", with an Undo
 * button for a fill that was a mistake. Undo puts the items back on the
 * list. Nothing shows until the backend has saved the shop. The page behind
 * re-reads the list when the sheet closes (`SendToWoolworths`).
 */
export function SavedPurchase({ handoff }: { handoff: TrolleyHandoff }) {
	const reported =
		handoff.status === "filled" || handoff.status === "filled_with_problems";
	const order = useSavedPurchase(handoff.id, reported);
	const { busyId, undo } = useUndoPurchase();
	const [undone, setUndone] = useState(false);

	if (!order) return null;
	if (undone) {
		return (
			<output className="text-sm">
				Undone. These items are back on your list.
			</output>
		);
	}

	const products = order.products ?? 0;
	return (
		<div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-border p-3 text-sm">
			<output>
				Saved as bought: {products} {products === 1 ? "product" : "products"},{" "}
				{formatMoney(order.total)}. These items left your list.
			</output>
			<Button
				size="sm"
				variant="outline"
				disabled={busyId === order.id}
				onClick={async () => setUndone(await undo(order))}
			>
				{busyId === order.id ? "Undoing…" : "Undo"}
			</Button>
		</div>
	);
}
