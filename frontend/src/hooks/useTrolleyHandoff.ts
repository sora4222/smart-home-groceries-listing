/**
 * Sends the chosen products to a store's trolley and follows the handoff
 * until the store tab reports back (or it is replaced).
 *
 * `send(delivery)` creates the handoff with the delivery time wanted; while one is open the hook re-reads it every
 * {@link POLL_MS} so the page shows each product's outcome as soon as the
 * bookmarklet reports. Polling stops when the handoff is finished, expired,
 * or the component unmounts.
 */
import { useCallback, useEffect, useState } from "react";

import { api } from "#/lib/api";
import type { StoreId } from "#/lib/api/products";
import {
	type DeliveryWanted,
	isHandoffFinished,
	type TrolleyHandoff,
} from "#/lib/api/trolley-handoffs";

/** How often an open handoff is re-read. */
export const POLL_MS = 3000;

export interface TrolleyHandoffState {
	handoff: TrolleyHandoff | null;
	sending: boolean;
	error: string | null;
	send: (delivery: DeliveryWanted) => Promise<TrolleyHandoff | null>;
}

export function useTrolleyHandoff(store: StoreId): TrolleyHandoffState {
	const [handoff, setHandoff] = useState<TrolleyHandoff | null>(null);
	const [sending, setSending] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const send = useCallback(
		async (delivery: DeliveryWanted) => {
			setSending(true);
			setError(null);
			try {
				const created = await api.trolleyHandoffs.create(store, delivery);
				setHandoff(created);
				return created;
			} catch (err) {
				console.error("[trolley-handoff] could not create", err);
				setError(messageFor(err));
				return null;
			} finally {
				setSending(false);
			}
		},
		[store],
	);

	const handoffId = handoff?.id;
	const open =
		handoff !== null && !isHandoffFinished(handoff) && !isExpired(handoff);
	useEffect(() => {
		if (!open || !handoffId) return;
		const timer = setInterval(async () => {
			try {
				setHandoff(await api.trolleyHandoffs.get(handoffId));
			} catch (err) {
				console.warn("[trolley-handoff] could not refresh", err);
			}
		}, POLL_MS);
		return () => clearInterval(timer);
	}, [open, handoffId]);

	return { handoff, sending, error, send };
}

/** A waiting handoff past its claim time can no longer be picked up. */
export function isExpired(handoff: TrolleyHandoff, now = Date.now()): boolean {
	return (
		handoff.status === "waiting_for_store_tab" &&
		new Date(handoff.expires_at).getTime() <= now
	);
}

/** The backend's `detail`, or a plain fallback. */
function messageFor(err: unknown): string {
	const body = (err as { body?: { detail?: unknown } })?.body;
	return typeof body?.detail === "string"
		? body.detail
		: "Could not send the list to the store — try again.";
}
