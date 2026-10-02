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
import { type DeliveryWanted, isHandoffFinished } from "#/lib/api";
import { abilitiesAt } from "#/lib/desktop/commands";

export { WOOLWORTHS_TROLLEY_URL } from "#/components/trolley/bookmark-steps";

/** Tomorrow (the store's next day), any time. */
export const DEFAULT_DELIVERY: DeliveryWanted = {
	date: null,
	time_of_day: "any",
};

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
 * The server cannot log in to Woolworths. In a browser the household's
 * logged-in Woolworths tab does the adding through the bookmark; in the
 * desktop app its own Woolworths window does (`FEATURE_DESKTOP_APP.md`).
 * Nothing is paid for here.
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
	useHandoffNotice(handoff.handoff);
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
			value={{ ...handoff, chosenCount, delivery, setDelivery }}
		>
			<Sheet onOpenChange={onOpenChange}>{children}</Sheet>
		</SendContext.Provider>
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

/** The steps (desktop app or bookmark), and the result once reported. */
function Content() {
	const { handoff, error } = useSend();
	const inApp = abilitiesAt(useDesktop(), "woolworths");
	const fillsInApp = inApp?.fill_trolley === true;

	return (
		<SheetContent className="flex flex-col gap-4 overflow-y-auto p-4">
			<SheetHeader className="p-0">
				<SheetTitle>Send to Woolworths</SheetTitle>
				<SheetDescription>
					Puts your chosen Woolworths products in your Woolworths trolley.{" "}
					{fillsInApp
						? "You pay at checkout in the Woolworths window."
						: "You pay on Woolworths."}
				</SheetDescription>
			</SheetHeader>
			{inApp && fillsInApp ? (
				<DesktopSteps abilities={inApp} />
			) : (
				<BookmarkSteps />
			)}
			{error && (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			)}
			{handoff && <HandoffStatus handoff={handoff} inApp={fillsInApp} />}
		</SheetContent>
	);
}

export const SendToWoolworths = Object.assign(SendToWoolworthsRoot, {
	Trigger,
	Content,
});
