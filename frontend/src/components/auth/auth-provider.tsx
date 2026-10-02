/**
 * Wraps the app in Clerk's provider when sign-in is on, and in nothing when
 * it is off. The one component that decides whether Clerk is on the page.
 */
import { ClerkProvider } from "@clerk/tanstack-react-start";
import type { ReactNode } from "react";

import { AUTH_MODE } from "#/lib/auth-mode";
import { SIGN_IN_PATH } from "#/lib/sign-in-guard";

export function AuthProvider({ children }: Readonly<{ children: ReactNode }>) {
	if (AUTH_MODE === "off") return children;
	return (
		<ClerkProvider
			signInUrl={SIGN_IN_PATH}
			signInFallbackRedirectUrl="/"
			afterSignOutUrl={SIGN_IN_PATH}
		>
			{children}
		</ClerkProvider>
	);
}
