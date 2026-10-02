import { DeliveryTimeChooser } from "#/components/trolley/delivery-time-chooser";
import { FillTrolleyBookmark } from "#/components/trolley/fill-trolley-bookmark";
import { useSend } from "#/components/trolley/send-context";
import { Button } from "#/components/ui/button";

/** Where the household's Woolworths trolley is. */
export const WOOLWORTHS_TROLLEY_URL =
	"https://www.woolworths.com.au/shop/mytrolley";

/**
 * The browser steps: drag the bookmark once, pick a delivery time, open
 * Woolworths, press the bookmark there.
 */
export function BookmarkSteps() {
	const { send, sending, delivery, setDelivery } = useSend();

	async function sendAndOpen() {
		// Open the tab now, while the click still counts, or it is blocked.
		const tab = window.open(WOOLWORTHS_TROLLEY_URL, "_blank");
		const created = await send(delivery);
		if (!created) tab?.close();
	}

	return (
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
	);
}
