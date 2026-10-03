/**
 * Auth token accessor — the one place the frontend reads a session token.
 *
 * With sign-in on, the token comes from Clerk: in the browser from the
 * loaded Clerk session, and during server rendering from the request's
 * session cookie (route loaders run on the server first). With sign-in off
 * (see `auth-mode.ts`) there is no token, and the backend must be running
 * with `DEV_AUTH_BYPASS=true`.
 *
 * Clerk is imported lazily so a build with sign-in off never loads it.
 */
import { createIsomorphicFn } from "@tanstack/react-start";

import { AUTH_MODE } from "#/lib/auth-mode";

/** The Clerk session token, read wherever this code is running. */
const clerkSessionToken = createIsomorphicFn()
	.server(async (): Promise<string | null> => {
		const { auth } = await import("@clerk/tanstack-react-start/server");
		return (await auth()).getToken();
	})
	.client(async (): Promise<string | null> => {
		const { getToken } = await import("@clerk/tanstack-react-start");
		return getToken();
	});

/** The current session token, or `null` when signed out or sign-in is off. */
export async function getAuthToken(): Promise<string | null> {
	if (AUTH_MODE === "off") return null;
	return clerkSessionToken();
}
