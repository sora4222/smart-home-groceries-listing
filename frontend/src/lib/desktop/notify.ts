/**
 * The one way the web app shows a desktop notification. Outside the desktop
 * app it does nothing; a refused notification is only logged.
 */
import { isDesktop } from "#/lib/desktop/bridge";
import { desktop } from "#/lib/desktop/commands";

/** A notification's title and body. */
export interface Notice {
	title: string;
	body: string;
}

/** Shows `notice` in the desktop app, without waiting for it. */
export function notifyDesktop(notice: Notice): void {
	if (!isDesktop()) return;
	desktop
		.notify(notice.title, notice.body)
		.catch((err) => console.warn("[desktop] notice failed", err));
}
