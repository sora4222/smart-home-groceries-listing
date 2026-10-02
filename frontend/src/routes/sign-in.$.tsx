/**
 * `/sign-in` — Clerk's sign-in box: Google, or email and password. The
 * splat (`$`) lets Clerk's own steps (verify email, second factor) live
 * under the same page. With sign-in off there is nothing to do here, so it
 * goes home.
 */
import { SignIn } from "@clerk/tanstack-react-start";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { AUTH_MODE } from "#/lib/auth-mode";
import { SIGN_IN_PATH } from "#/lib/sign-in-guard";

export const Route = createFileRoute("/sign-in/$")({
	beforeLoad: () => {
		if (AUTH_MODE === "off") throw redirect({ to: "/" });
	},
	head: () => ({ meta: [{ title: "Sign in · Grocery List" }] }),
	component: SignInPage,
});

function SignInPage() {
	return (
		<div className="flex justify-center py-8">
			<SignIn path={SIGN_IN_PATH} routing="path" fallbackRedirectUrl="/" />
		</div>
	);
}
