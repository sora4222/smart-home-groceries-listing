import { useRouter } from "@tanstack/react-router";
import { type ReactNode, useState } from "react";

import { BookmarkSteps } from "#/components/trolley/bookmark-steps";
import { DesktopSteps } from "#/components/trolley/desktop-steps";
import { HandoffStatus } from "#/components/trolley/handoff-status";
import { SendContext, useSend } from "#/components/trolley/send-context";
import { Button } from "#/components/ui/button";
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "#/components/ui/sheet";
import { useDesktop } from "#/hooks/useDesktop";
import { useHandoffNotice } from "#/hooks/useHandoffNotice";
import { useTrolleyHandoff } from "#/hooks/useTrolleyHandoff";
import {
	type DeliveryWanted,
	isHandoffFinished,
	type StoreId,
} from "#/lib/api";
import { abilitiesAt } from "#/lib/desktop/commands";
import { STORE_TABS } from "#/lib/store-tab/store-tabs";

/** Tomorrow (the store's next day), any time. */
export const DEFAULT_DELIVERY: DeliveryWanted = {
	date: null,
	time_of_day: "any",
};

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
	useHandoffNotice(handoff.handoff);
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
		<SendContext.Provider
			value={{
				...handoff,
				tab: STORE_TABS[store],
				chosenCount,
				delivery,
				setDelivery,
			}}
		>
			<Sheet onOpenChange={onOpenChange}>{children}</Sheet>
		</SendContext.Provider>
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
	const { tab, handoff, error } = useSend();
	const name = tab.storeName;
	const abilities = abilitiesAt(useDesktop(), tab.store);
	const inApp = abilities?.fill_trolley === true;

	return (
		<SheetContent className="flex flex-col gap-4 overflow-y-auto p-4">
			<SheetHeader className="p-0">
				<SheetTitle>Send to {name}</SheetTitle>
				<SheetDescription>
					Puts your chosen {name} products in your {name} trolley. You pay on{" "}
					{name}.
				</SheetDescription>
			</SheetHeader>
			{inApp && abilities ? (
				<DesktopSteps abilities={abilities} />
			) : (
				<BookmarkSteps />
			)}

			{error && (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			)}
			{handoff && <HandoffStatus handoff={handoff} inApp={inApp} />}
		</SheetContent>
	);
}

export const SendToStore = Object.assign(SendToStoreRoot, {
	Trigger,
	Content,
});
