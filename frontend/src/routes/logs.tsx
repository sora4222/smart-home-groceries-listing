import { createFileRoute, redirect } from "@tanstack/react-router";

/** `/logs` — the old Logs page, now Settings › Logs. */
export const Route = createFileRoute("/logs")({
	beforeLoad: () => {
		throw redirect({ to: "/settings/logs", replace: true });
	},
});
