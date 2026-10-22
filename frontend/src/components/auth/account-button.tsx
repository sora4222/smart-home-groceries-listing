/**
 * The signed-in person's avatar in the nav: opens Clerk's menu with
 * "Manage account" and "Sign out". Renders nothing with sign-in off.
 */
import { UserButton } from "@clerk/tanstack-react-start";

import { AUTH_MODE } from "#/lib/auth-mode";

export function AccountButton() {
	if (AUTH_MODE === "off") return null;
	return (
		<div className="ml-auto flex shrink-0 items-center pl-2">
			<UserButton />
		</div>
	);
}
