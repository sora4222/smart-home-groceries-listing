import { createFileRoute } from "@tanstack/react-router";

import { GoogleTasksSection } from "#/components/intake/google-tasks-section";
import { OtherChannels } from "#/components/intake/other-channels";
import {
	type GoogleReturn,
	useGoogleSignInReturn,
} from "#/hooks/useGoogleSignInReturn";
import { api } from "#/lib/api";

/** Keeps only Google's answer from the address, as strings. */
function googleReturn(search: Record<string, unknown>): GoogleReturn {
	const pick = (key: string) =>
		typeof search[key] === "string" ? (search[key] as string) : undefined;
	const found: GoogleReturn = {};
	for (const key of ["code", "state", "error"] as const) {
		const value = pick(key);
		if (value) found[key] = value;
	}
	return found;
}

export const Route = createFileRoute("/settings/intake")({
	validateSearch: googleReturn,
	loader: () => api.intake.settings(),
	component: IntakeSettingsPage,
});

/**
 * `/settings/intake` — the ways items get in (spec: "Intake channel
 * configuration"). Google Tasks is set up here; Alexa, the webhook and the
 * item checker are set in the server's `.env` and only shown.
 *
 * This page is also where Google sends the browser back after sign-in.
 */
function IntakeSettingsPage() {
	const settings = Route.useLoaderData();
	useGoogleSignInReturn(Route.useSearch());
	const tasks = settings.google_tasks;

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Intake</h1>
				<p className="text-sm text-muted-foreground">
					How items get to Pending Requests. Nothing goes on the list until
					someone accepts it.
				</p>
			</div>
			<GoogleTasksSection
				key={`${tasks.connected_at}-${tasks.task_list?.id}`}
				initial={tasks}
			/>
			<OtherChannels settings={settings} />
		</div>
	);
}
