import { useCountRiseToast } from "#/hooks/useCountRiseToast";
import { isDesktop } from "#/lib/desktop/bridge";
import { desktop } from "#/lib/desktop/commands";
import type { Notice } from "#/lib/desktop/handoff-notice";

/** The notice for a new item reaching Pending Requests. */
export const NEW_ITEM_NOTICE: Notice = {
	title: "New grocery item",
	body: "An item is waiting for you to accept it.",
};

/** The notice for an item triage held for review. */
export const HELD_ITEM_NOTICE: Notice = {
	title: "Item held for review",
	body: "Open Triage to accept or reject it.",
};

function send(notice: Notice) {
	if (!isDesktop()) return;
	desktop
		.notify(notice.title, notice.body)
		.catch((err) => console.warn("[desktop] notice failed", err));
}

/**
 * In the desktop app, adds a desktop notification next to the in-page toast
 * when a new item arrives or one is held for review. Mount once, at the root
 * layout. Does nothing in a browser.
 */
export function useDesktopNotifications(): void {
	useCountRiseToast("voice_request_added", () => send(NEW_ITEM_NOTICE));
	useCountRiseToast("triage_held", () => send(HELD_ITEM_NOTICE));
}
