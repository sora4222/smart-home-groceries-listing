import { createFileRoute, redirect } from "@tanstack/react-router";

import { googleReturn } from "#/lib/google-return";

/**
 * `/settings/intake` — the old Intake page, now Settings › Help. Kept because
 * Google's sign-in returns here (`GOOGLE_REDIRECT_URI`); its answer is passed on.
 */
export const Route = createFileRoute("/settings/intake")({
	validateSearch: googleReturn,
	beforeLoad: ({ search }) => {
		throw redirect({ to: "/settings/help", search, replace: true });
	},
});
