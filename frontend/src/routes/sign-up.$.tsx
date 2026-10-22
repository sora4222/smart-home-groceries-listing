/**
 * `/sign-up` — Clerk's sign-up box, for a household member's first visit.
 * Clerk's allowlist decides who may finish it. The splat (`$`) lets Clerk's
 * own steps (verify email) live under the same page. With sign-in off there
 * is nothing to do here, so it goes home.
 */
import { SignUp } from "@clerk/tanstack-react-start";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { AUTH_MODE } from "#/lib/auth-mode";
import { SIGN_IN_PATH, SIGN_UP_PATH } from "#/lib/sign-in-guard";

export const Route = createFileRoute("/sign-up/$")({
	beforeLoad: () => {
		if (AUTH_MODE === "off") throw redirect({ to: "/" });
	},
	head: () => ({ meta: [{ title: "Sign up · Grocery List" }] }),
	component: SignUpPage,
});

function SignUpPage() {
	return (
		<div className="flex justify-center py-8">
			<SignUp
				path={SIGN_UP_PATH}
				routing="path"
				signInUrl={SIGN_IN_PATH}
				fallbackRedirectUrl="/"
			/>
		</div>
	);
}
