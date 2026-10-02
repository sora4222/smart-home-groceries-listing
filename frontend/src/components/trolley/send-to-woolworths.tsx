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
import { type DeliveryWanted, isHandoffFinished } from "#/lib/api";

/** Tomorrow (the store's next day), any time. */
export const DEFAULT_DELIVERY: DeliveryWanted = {
	date: null,
	time_of_day: "any",
};

/** Where the household's Woolworths trolley is. */
export const WOOLWORTHS_TROLLEY_URL =
	"https://www.woolworths.com.au/shop/mytrolley";

interface SendContext extends TrolleyHandoffState {
	/** How many list items have a Woolworths product chosen. */
	chosenCount: number;
	/** The delivery time to reserve; tomorrow, any time, until changed. */
	delivery: DeliveryWanted;
	setDelivery: (delivery: DeliveryWanted) => void;
}

const Context = createContext<SendContext | null>(null);

function useSend(): SendContext {
	const value = useContext(Context);
	if (!value)
		throw new Error("SendToWoolworths parts must be inside <SendToWoolworths>");
	return value;
}

/**
 * Puts every product chosen at Woolworths into the household's own
 * Woolworths trolley, in a sheet, after reserving a delivery time (default:
 * tomorrow, any time — changeable on Woolworths later).
 *
 * ```tsx
 * <SendToWoolworths chosenCount={3}>
 *   <SendToWoolworths.Trigger />
 *   <SendToWoolworths.Content />
 * </SendToWoolworths>
 * ```
 *
 * The server cannot log in to Woolworths, so the household's logged-in
 * Woolworths tab does the adding, through the "Fill Woolworths trolley"
 * bookmark. Nothing is paid for here.
 */
function SendToWoolworthsRoot({
	chosenCount,
	children,
}: {
	chosenCount: number;
	children: ReactNode;
}) {
	const handoff = useTrolleyHandoff("woolworths");
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
			value={{ ...handoff, chosenCount, delivery, setDelivery }}
		>
			<Sheet onOpenChange={onOpenChange}>{children}</Sheet>
		</Context.Provider>
	);
}

/** The button that opens the sheet; disabled until something is chosen. */
function Trigger() {
	const { chosenCount } = useSend();
	return (
		<SheetTrigger asChild>
			<Button
				variant="outline"
				className="w-full sm:w-auto"
				disabled={chosenCount === 0}
			>
				Send to Woolworths ({chosenCount} {chosenCount === 1 ? "item" : "items"}
				)
			</Button>
		</SheetTrigger>
	);
}

/** The three steps, and the result once the bookmark reports back. */
function Content() {
	const { send, sending, handoff, error, delivery, setDelivery } = useSend();

	async function sendAndOpen() {
		// Open the tab now, while the click still counts, or it is blocked.
		const tab = window.open(WOOLWORTHS_TROLLEY_URL, "_blank");
		const created = await send(delivery);
		if (!created) tab?.close();
	}

	return (
		<SheetContent className="flex flex-col gap-4 overflow-y-auto p-4">
			<SheetHeader className="p-0">
				<SheetTitle>Send to Woolworths</SheetTitle>
				<SheetDescription>
					Puts your chosen Woolworths products in your Woolworths trolley. You
					pay on Woolworths.
				</SheetDescription>
			</SheetHeader>
			<ol className="flex list-decimal flex-col gap-3 pl-5 text-sm">
				<li>
					<p>Only once: drag this to your bookmarks bar.</p>
					<FillTrolleyBookmark />
				</li>
				<li>
					<p>Pick a delivery time. You can change it on Woolworths later.</p>
					<div className="mt-1">
						<DeliveryTimeChooser value={delivery} onChange={setDelivery} />
					</div>
				</li>
				<li>
					<p>Press the button. Woolworths opens. Log in if asked.</p>
					<Button
						className="mt-1 w-full sm:w-auto"
						disabled={sending}
						onClick={sendAndOpen}
					>
						{sending ? "Sending…" : "Send and open Woolworths"}
					</Button>
				</li>
				<li>On Woolworths, press the “Fill Woolworths trolley” bookmark.</li>
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

export const SendToWoolworths = Object.assign(SendToWoolworthsRoot, {
	Trigger,
	Content,
});
