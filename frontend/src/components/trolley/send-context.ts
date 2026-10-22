/**
 * The shared state inside `<SendToStore>`: the handoff, the delivery
 * time wanted, and how many items are chosen. Its parts read it with
 * {@link useSend}.
 */
import { createContext, useContext } from "react";

import type { TrolleyHandoffState } from "#/hooks/useTrolleyHandoff";
import type { DeliveryWanted } from "#/lib/api";

import type { StoreTab } from "#/lib/store-tab/store-tab";

export interface SendContextValue extends TrolleyHandoffState {
	tab: StoreTab;
	/** How many list items have a store product chosen. */
	chosenCount: number;
	/** The delivery time to reserve; tomorrow, any time, until changed. */
	delivery: DeliveryWanted;
	setDelivery: (delivery: DeliveryWanted) => void;
}

export const SendContext = createContext<SendContextValue | null>(null);

/** The sheet's state. Only for parts inside `<SendToStore>`. */
export function useSend(): SendContextValue {
	const value = useContext(SendContext);
	if (!value) throw new Error("SendToStore parts must be inside <SendToStore>");
	return value;
}
