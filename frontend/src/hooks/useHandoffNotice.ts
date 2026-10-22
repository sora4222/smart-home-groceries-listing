import { useEffect, useRef } from "react";

import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";
import { handoffNotice } from "#/lib/desktop/handoff-notice";
import { notifyDesktop } from "#/lib/desktop/notify";

/**
 * In the desktop app, sends one notification when `handoff` is filled — so a
 * person who looked away from the sheet hears that the trolley is ready.
 * Each handoff is announced once.
 */
export function useHandoffNotice(handoff: TrolleyHandoff | null): void {
	const announced = useRef<string | null>(null);
	useEffect(() => {
		if (!handoff || announced.current === handoff.id) return;
		const notice = handoffNotice(handoff);
		if (!notice) return;
		announced.current = handoff.id;
		notifyDesktop(notice);
	}, [handoff]);
}
