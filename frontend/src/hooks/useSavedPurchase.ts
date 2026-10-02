/**
 * The shop saved from one trolley fill, once the backend has saved it.
 *
 * The backend saves a filled trolley as bought a moment after the bookmark
 * reports, so this re-reads the saved shops every {@link SAVED_POLL_MS} until
 * the one for `handoffId` appears, at most {@link SAVED_POLL_TRIES} times.
 * `null` until then — and for good when nothing was added to the trolley.
 */
import { useEffect, useState } from "react";

import { api, type PurchaseOrder } from "#/lib/api";

export const SAVED_POLL_MS = 1500;
export const SAVED_POLL_TRIES = 10;

export function useSavedPurchase(
	handoffId: string,
	reported: boolean,
): PurchaseOrder | null {
	const [order, setOrder] = useState<PurchaseOrder | null>(null);

	useEffect(() => {
		if (!reported) return;
		let tries = 0;
		let timer: ReturnType<typeof setTimeout> | undefined;
		let current = true;
		const look = async () => {
			tries += 1;
			try {
				const found = (await api.purchases.listOrders()).find(
					(o) => o.trolley_handoff_id === handoffId,
				);
				if (!current) return;
				if (found) {
					setOrder(found);
					return;
				}
			} catch (err) {
				console.warn("[purchases] could not read saved shops", err);
			}
			if (current && tries < SAVED_POLL_TRIES) {
				timer = setTimeout(look, SAVED_POLL_MS);
			}
		};
		look();
		return () => {
			current = false;
			clearTimeout(timer);
		};
	}, [handoffId, reported]);

	return order;
}
