import { DeliveryTimeChooser } from "#/components/trolley/delivery-time-chooser";
import { FillTrolleyBookmark } from "#/components/trolley/fill-trolley-bookmark";
import { useSend } from "#/components/trolley/send-context";
import { Button } from "#/components/ui/button";

/**
 * The browser steps: drag the bookmark once, pick a delivery time, open
 * Woolworths, press the bookmark there.
 */
export function BookmarkSteps() {
	const { tab, send, sending, delivery, setDelivery } = useSend();
	const name = tab.storeName;

	async function sendAndOpen() {
		// Open the tab now, while the click still counts, or it is blocked.
		const opened = window.open(tab.trolleyUrl, "_blank");
		const created = await send(delivery);
		if (!created) opened?.close();
	}

	return (
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
				On {name}, press the "{tab.bookmarkName}" bookmark.
				{!tab.reservesDelivery && ` Then pick a delivery time on ${name}.`}
			</li>
		</ol>
	);
}
