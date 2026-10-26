import { createFileRoute } from "@tanstack/react-router";

import { GoogleTasksSection } from "#/components/intake/google-tasks-section";
import { OtherChannels } from "#/components/intake/other-channels";
import { useGoogleSignInReturn } from "#/hooks/useGoogleSignInReturn";
import { api } from "#/lib/api";
import { googleReturn } from "#/lib/google-return";

export const Route = createFileRoute("/settings/help")({
	validateSearch: googleReturn,
	loader: () => api.intake.settings(),
	component: HelpPage,
});

/**
 * Settings › Help — how items get in (spec: "Intake channel configuration").
 * Google Tasks is set up here; Alexa, the webhook and the item checker are
 * set in the server's `.env` and only shown.
 *
 * Google's sign-in returns to `/settings/intake`, which forwards here.
 */
function HelpPage() {
	const settings = Route.useLoaderData();
	useGoogleSignInReturn(Route.useSearch());
	const tasks = settings.google_tasks;

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Help</h1>
				<p className="text-sm text-muted-foreground">
					Ways to add items. Each one waits in Pending until someone accepts it.
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
