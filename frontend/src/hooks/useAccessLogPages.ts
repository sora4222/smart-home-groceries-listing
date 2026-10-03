import { useState } from "react";
import { toast } from "sonner";

import { type AccessLogEntry, type AccessLogPage, api } from "#/lib/api";

/**
 * The `/logs` page's rows: the first page as loaded, "Load older" to append
 * the next page, and the health-check switch, which starts again from the
 * newest rows. A failed load keeps what is shown and says so.
 */
export function useAccessLogPages(initial: AccessLogPage) {
	const [entries, setEntries] = useState<AccessLogEntry[]>(initial.entries);
	const [nextBefore, setNextBefore] = useState(initial.next_before);
	const [showHealthChecks, setShowHealthChecks] = useState(false);
	const [loading, setLoading] = useState(false);

	async function load(before: number | undefined, showHealth: boolean) {
		setLoading(true);
		try {
			const page = await api.accessLogs.list({
				before,
				hideHealthChecks: !showHealth,
			});
			setEntries((prev) =>
				before === undefined ? page.entries : [...prev, ...page.entries],
			);
			setNextBefore(page.next_before);
			return true;
		} catch {
			toast.error("Could not load the access log — try again.");
			return false;
		} finally {
			setLoading(false);
		}
	}

	const loadOlder = () => {
		if (nextBefore !== null) void load(nextBefore, showHealthChecks);
	};

	const changeHealthChecks = async (show: boolean) => {
		if (await load(undefined, show)) setShowHealthChecks(show);
	};

	return {
		entries,
		hasOlder: nextBefore !== null,
		loading,
		showHealthChecks,
		loadOlder,
		changeHealthChecks,
	};
}
