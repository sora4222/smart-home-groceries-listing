/**
 * Where a page visit should go, given whether sign-in is on and whether the
 * visitor is signed in. Pure, so the rules are unit-tested without Clerk.
 */
import type { AuthMode } from "#/lib/auth-mode";

/** The sign-in page's path. Its sub-paths belong to Clerk's own steps. */
export const SIGN_IN_PATH = "/sign-in";

/** `true` for the sign-in page and every step under it. */
export function isSignInPath(pathname: string): boolean {
	return pathname === SIGN_IN_PATH || pathname.startsWith(`${SIGN_IN_PATH}/`);
}

/** Whether the guard needs to ask the server about the session at all. */
export function needsSessionCheck(mode: AuthMode, pathname: string): boolean {
	return mode === "clerk" && !isSignInPath(pathname);
}

/** `"sign-in"` when a visitor must sign in first, `"stay"` otherwise. */
export function guardDecision(
	mode: AuthMode,
	pathname: string,
	signedIn: boolean,
): "sign-in" | "stay" {
	if (!needsSessionCheck(mode, pathname)) return "stay";
	return signedIn ? "stay" : "sign-in";
}
