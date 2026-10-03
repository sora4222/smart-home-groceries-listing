import { createFileRoute } from "@tanstack/react-router";

import { AccessLogList } from "#/components/access-logs/access-log-list";
import { HealthChecksSwitch } from "#/components/access-logs/health-checks-switch";
import { Button } from "#/components/ui/button";
import { useAccessLogPages } from "#/hooks/useAccessLogPages";
import { api } from "#/lib/api";

export const Route = createFileRoute("/logs")({
	loader: () => api.accessLogs.list({ hideHealthChecks: true }),
	component: LogsPage,
});

/**
 * `/logs` — every request the backend answered, newest first (spec: "Network
 * access logging"). Any signed-in household member may read it.
 */
function LogsPage() {
	const {
		entries,
		hasOlder,
		loading,
		showHealthChecks,
		loadOlder,
		changeHealthChecks,
	} = useAccessLogPages(Route.useLoaderData());

	return (
		<div className="flex flex-col gap-3">
			<h1 className="text-lg font-semibold">Access log</h1>
			<p className="text-sm text-muted-foreground">
				Every request to the app: when, who, from where, and what it asked for.
				Newest first.
			</p>
			<HealthChecksSwitch
				checked={showHealthChecks}
				onCheckedChange={changeHealthChecks}
			/>
			<AccessLogList entries={entries} />
			{hasOlder && (
				<Button
					variant="outline"
					className="self-center"
					disabled={loading}
					onClick={loadOlder}
				>
					Load older
				</Button>
			)}
		</div>
	);
}
