import { useState } from "react";

import { DeliveryTimeChooser } from "#/components/trolley/delivery-time-chooser";
import { useSend } from "#/components/trolley/send-context";
import { Button } from "#/components/ui/button";
import { useFillTrolleyConfig } from "#/hooks/useFillTrolleyConfig";
import { isHandoffFinished } from "#/lib/api";
import { desktop, type StoreAbilities } from "#/lib/desktop/commands";
import { buildFillWoolworthsTrolleyProgram } from "#/lib/store-tab/bookmarklet";

/**
 * The desktop app's steps: no bookmark. Log in once in the app's Woolworths
 * window, pick a delivery time, press **Fill trolley in the app**. Once
 * filled, **Open checkout** shows the checkout page to check and pay.
 */
export function DesktopSteps({ abilities }: { abilities: StoreAbilities }) {
	const { send, sending, handoff, delivery, setDelivery } = useSend();
	const { config, failed } = useFillTrolleyConfig();
	const [problem, setProblem] = useState<string | null>(null);

	async function run(action: () => Promise<void>) {
		setProblem(null);
		try {
			await action();
		} catch (err) {
			setProblem(err instanceof Error ? err.message : String(err));
		}
	}

	async function fillInApp() {
		if (!config) return;
		const created = await send(delivery);
		if (!created) return;
		await run(() =>
			desktop.fillTrolley(
				"woolworths",
				buildFillWoolworthsTrolleyProgram(config, "log"),
			),
		);
	}

	const filled = handoff !== null && isHandoffFinished(handoff);
	return (
		<>
			<ol className="flex list-decimal flex-col gap-3 pl-5 text-sm">
				<li>
					<p>Only once: log in to Woolworths in the app. It remembers you.</p>
					<Button
						variant="outline"
						className="mt-1 w-full sm:w-auto"
						disabled={!abilities.login}
						onClick={() => run(() => desktop.openStore("woolworths"))}
					>
						Log in to Woolworths
					</Button>
				</li>
				<li>
					<p>Pick a delivery time. You can change it on Woolworths later.</p>
					<div className="mt-1">
						<DeliveryTimeChooser value={delivery} onChange={setDelivery} />
					</div>
				</li>
				<li>
					<p>Press the button. A Woolworths window fills your trolley.</p>
					<Button
						className="mt-1 w-full sm:w-auto"
						disabled={sending || !config}
						onClick={fillInApp}
					>
						{sending ? "Sending…" : "Fill trolley in the app"}
					</Button>
					{failed && (
						<p className="text-destructive">
							Cannot fill: STORE_TAB_SECRET is not set on the server.
						</p>
					)}
				</li>
				{filled && abilities.checkout && (
					<li>
						<p>Check the trolley and pay at checkout.</p>
						<Button
							className="mt-1 w-full sm:w-auto"
							onClick={() => run(() => desktop.openCheckout("woolworths"))}
						>
							Open checkout
						</Button>
					</li>
				)}
			</ol>
			{problem && (
				<p role="alert" className="text-sm text-destructive">
					{problem}
				</p>
			)}
		</>
	);
}
