/**
 * Wraps the app in Clerk's provider when sign-in is on, and in nothing when
 * it is off. Clerk's boxes use the shadcn theme, so they match the app.
 * The one component that decides whether Clerk is on the page.
 */
import { ClerkProvider } from "@clerk/tanstack-react-start";
import { shadcn } from "@clerk/ui/themes";
import type { ReactNode } from "react";

import { AUTH_MODE } from "#/lib/auth-mode";
import { SIGN_IN_PATH, SIGN_UP_PATH } from "#/lib/sign-in-guard";

export function AuthProvider({ children }: Readonly<{ children: ReactNode }>) {
	if (AUTH_MODE === "off") return children;
	return (
		<ClerkProvider
			appearance={{ theme: shadcn }}
			signInUrl={SIGN_IN_PATH}
			signUpUrl={SIGN_UP_PATH}
			signInFallbackRedirectUrl="/"
			signUpFallbackRedirectUrl="/"
			afterSignOutUrl={SIGN_IN_PATH}
		>
			{children}
		</ClerkProvider>
	);
}
