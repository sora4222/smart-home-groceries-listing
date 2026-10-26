import { createFileRoute, redirect } from "@tanstack/react-router";

import { validateAnalysisSearch } from "#/lib/analysis-search";

/** `/analysis` — the old Spending page, now Order › History. */
export const Route = createFileRoute("/analysis")({
	validateSearch: validateAnalysisSearch,
	beforeLoad: ({ search }) => {
		throw redirect({ to: "/order/history", search, replace: true });
	},
});
