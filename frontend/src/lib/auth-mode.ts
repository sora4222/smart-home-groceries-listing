/**
 * Whether the web app asks people to sign in.
 *
 * - `clerk`: sign-in with Clerk. Chosen when `VITE_CLERK_PUBLISHABLE_KEY` is
 *   set.
 * - `off`: no sign-in screen and no session token. For the e2e tests and
 *   local work, with the backend's `DEV_AUTH_BYPASS=true`. Chosen when there
 *   is no publishable key, or forced with `VITE_AUTH_MODE=off` (so a real key
 *   in `.env` does not get in the way of the tests).
 *
 * The backend is the real gate either way: with sign-in off and the bypass
 * off, every request is refused.
 */
export type AuthMode = "clerk" | "off";

/** The environment values the mode is decided from. */
export interface AuthModeEnv {
	VITE_AUTH_MODE?: string;
	VITE_CLERK_PUBLISHABLE_KEY?: string;
}

/** Decides the mode from the environment. Pure, so it can be unit-tested. */
export function resolveAuthMode(env: AuthModeEnv): AuthMode {
	if (env.VITE_AUTH_MODE?.trim().toLowerCase() === "off") return "off";
	return env.VITE_CLERK_PUBLISHABLE_KEY?.trim() ? "clerk" : "off";
}

/** This build's mode. */
export const AUTH_MODE: AuthMode = resolveAuthMode(import.meta.env);
