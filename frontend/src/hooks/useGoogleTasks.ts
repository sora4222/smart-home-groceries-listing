import { useEffect, useState } from "react";
import { toast } from "sonner";

import {
	api,
	type GoogleTasksChoices,
	type GoogleTasksStatus,
	type TaskList,
} from "#/lib/api";
import { pollReportMessage } from "#/lib/poll-report";

/** Which Google Tasks action is running, if any. */
export type GoogleTasksBusy =
	| "connect"
	| "save"
	| "check"
	| "disconnect"
	| null;

/** The detail of a backend refusal, when it is a sentence for a person. */
function detailOf(error: unknown): string | null {
	const body = (error as { body?: { detail?: unknown } })?.body;
	return typeof body?.detail === "string" ? body.detail : null;
}

/**
 * The Google Tasks section's state and actions: connect (off to Google's
 * sign-in page), load the account's lists, save choices, check now,
 * disconnect. Every failure is a toast; nothing changes on screen that the
 * backend did not accept.
 */
export function useGoogleTasks(initial: GoogleTasksStatus) {
	const [status, setStatus] = useState(initial);
	const [lists, setLists] = useState<TaskList[] | null>(null);
	const [busy, setBusy] = useState<GoogleTasksBusy>(null);

	useEffect(() => {
		if (!status.connected) return;
		let live = true;
		api.intake
			.lists()
			.then((found) => live && setLists(found))
			.catch((error) => {
				if (live)
					toast.error(
						detailOf(error) ?? "Could not load your Google Tasks lists.",
					);
			});
		return () => {
			live = false;
		};
	}, [status.connected]);

	async function run<T>(
		kind: GoogleTasksBusy,
		call: () => Promise<T>,
		failure: string,
	) {
		setBusy(kind);
		try {
			return await call();
		} catch (error) {
			toast.error(detailOf(error) ?? failure);
			return undefined;
		} finally {
			setBusy(null);
		}
	}

	async function connect() {
		const started = await run(
			"connect",
			api.intake.startSignIn,
			"Could not start the Google sign-in.",
		);
		if (started) window.location.assign(started.authorize_url);
	}

	async function save(choices: GoogleTasksChoices) {
		const saved = await run(
			"save",
			() => api.intake.saveChoices(choices),
			"Could not save — try again.",
		);
		if (saved) {
			setStatus(saved);
			toast.success("Google Tasks settings saved");
		}
	}

	async function checkNow() {
		const report = await run(
			"check",
			api.intake.pollNow,
			"Could not check Google Tasks.",
		);
		if (report) {
			toast.success(pollReportMessage(report));
			const fresh = await api.intake.settings().catch(() => null);
			if (fresh) setStatus(fresh.google_tasks);
		}
	}

	async function disconnect() {
		const done = await run(
			"disconnect",
			async () => {
				await api.intake.disconnect();
				return true;
			},
			"Could not disconnect — try again.",
		);
		if (done) {
			setStatus((prev) => ({
				...prev,
				connected: false,
				connected_at: null,
				enabled: false,
				polling: false,
			}));
			setLists(null);
			toast("Google Tasks disconnected");
		}
	}

	return { status, lists, busy, connect, save, checkNow, disconnect };
}
