import { useRouter } from "@tanstack/react-router";
import { createContext, type ReactNode, useContext, useState } from "react";

import { DeliveryTimeChooser } from "#/components/trolley/delivery-time-chooser";
import { FillTrolleyBookmark } from "#/components/trolley/fill-trolley-bookmark";
import { HandoffStatus } from "#/components/trolley/handoff-status";
import { Button } from "#/components/ui/button";
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "#/components/ui/sheet";
import {
	type TrolleyHandoffState,
	useTrolleyHandoff,
} from "#/hooks/useTrolleyHandoff";
import {
	type DeliveryWanted,
	isHandoffFinished,
	type StoreId,
} from "#/lib/api";
import type { StoreTab } from "#/lib/store-tab/store-tab";
import { STORE_TABS } from "#/lib/store-tab/store-tabs";

/** Tomorrow (the store's next day), any time. */
export const DEFAULT_DELIVERY: DeliveryWanted = {
	date: null,
	time_of_day: "any",
};

interface SendContext extends TrolleyHandoffState {
	/** The store's trolley handoff: names, links and its bookmark. */
	tab: StoreTab;
	/** How many list items have a product chosen at this store. */
	chosenCount: number;
	/** The delivery time to reserve; tomorrow, any time, until changed. */
	delivery: DeliveryWanted;
	setDelivery: (delivery: DeliveryWanted) => void;
}

const Context = createContext<SendContext | null>(null);

function useSend(): SendContext {
	const value = useContext(Context);
	if (!value) throw new Error("SendToStore parts must be inside <SendToStore>");
	return value;
}

/**
 * Puts every product chosen at one store into the household's own trolley
 * at that store, in a sheet. Where the store's bookmark can, it reserves a
 * delivery time first (default: tomorrow, any time — changeable on the
 * store's website later).
 *
 * ```tsx
 * <SendToStore store="coles" chosenCount={3}>
 *   <SendToStore.Trigger />
 *   <SendToStore.Content />
 * </SendToStore>
 * ```
 *
 * The server cannot log in to a store, so the household's logged-in tab does
 * the adding, through the store's "Fill … trolley" bookmark
 * (`lib/store-tab/store-tabs.ts`). Nothing is paid for here.
 */
function SendToStoreRoot({
	store,
	chosenCount,
	children,
}: {
	store: StoreId;
	chosenCount: number;
	children: ReactNode;
}) {
	const handoff = useTrolleyHandoff(store);
	const [delivery, setDelivery] = useState<DeliveryWanted>(DEFAULT_DELIVERY);
	const router = useRouter({ warn: false });
	// A filled trolley is saved as bought and its items leave the list (or
	// come back after Undo). Re-read the page once the sheet closes — not
	// while it is open, which could remove the card the sheet belongs to.
	function onOpenChange(open: boolean) {
		if (!open && handoff.handoff && isHandoffFinished(handoff.handoff)) {
			router?.invalidate();
		}
	}
	return (
		<Context.Provider
			value={{
				...handoff,
				tab: STORE_TABS[store],
				chosenCount,
				delivery,
				setDelivery,
			}}
		>
			<Sheet onOpenChange={onOpenChange}>{children}</Sheet>
		</Context.Provider>
	);
}

/** The button that opens the sheet; disabled until something is chosen. */
function Trigger() {
	const { tab, chosenCount } = useSend();
	return (
		<SheetTrigger asChild>
			<Button
				variant="outline"
				className="w-full sm:w-auto"
				disabled={chosenCount === 0}
			>
				Send to {tab.storeName} ({chosenCount}{" "}
				{chosenCount === 1 ? "item" : "items"})
			</Button>
		</SheetTrigger>
	);
}

/** The steps, and the result once the bookmark reports back. */
function Content() {
	const { tab, send, sending, handoff, error, delivery, setDelivery } =
		useSend();
	const name = tab.storeName;

	async function sendAndOpen() {
		// Open the tab now, while the click still counts, or it is blocked.
		const opened = window.open(tab.trolleyUrl, "_blank");
		const created = await send(delivery);
		if (!created) opened?.close();
	}

	return (
		<SheetContent className="flex flex-col gap-4 overflow-y-auto p-4">
			<SheetHeader className="p-0">
				<SheetTitle>Send to {name}</SheetTitle>
				<SheetDescription>
					Puts your chosen {name} products in your {name} trolley. You pay on{" "}
					{name}.
				</SheetDescription>
			</SheetHeader>
			<ol className="flex list-decimal flex-col gap-3 pl-5 text-sm">
				<li>
					<p>Only once: drag this to your bookmarks bar.</p>
					<FillTrolleyBookmark tab={tab} />
				</li>
				{tab.reservesDelivery && (
					<li>
						<p>Pick a delivery time. You can change it on {name} later.</p>
						<div className="mt-1">
							<DeliveryTimeChooser value={delivery} onChange={setDelivery} />
						</div>
					</li>
				)}
				<li>
					<p>Press the button. {name} opens. Log in if asked.</p>
					<Button
						className="mt-1 w-full sm:w-auto"
						disabled={sending}
						onClick={sendAndOpen}
					>
						{sending ? "Sending…" : `Send and open ${name}`}
					</Button>
				</li>
				<li>
					On {name}, press the “{tab.bookmarkName}” bookmark.
					{!tab.reservesDelivery && ` Then pick a delivery time on ${name}.`}
				</li>
			</ol>
			{error && (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			)}
			{handoff && <HandoffStatus handoff={handoff} />}
		</SheetContent>
	);
}

export const SendToStore = Object.assign(SendToStoreRoot, {
	Trigger,
	Content,
});
