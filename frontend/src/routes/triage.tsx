import { createFileRoute, redirect } from "@tanstack/react-router";

/** `/triage` — the old Triage page, now the lower half of Pending. */
export const Route = createFileRoute("/triage")({
	validateSearch: (search: Record<string, unknown>): { tab?: "rejected" } =>
		search.tab === "rejected" ? { tab: "rejected" } : {},
	beforeLoad: ({ search }) => {
		throw redirect({ to: "/pending", search, replace: true });
	},
});
