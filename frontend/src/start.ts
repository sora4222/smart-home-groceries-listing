/**
 * The TanStack Start instance. With sign-in on, Clerk's middleware reads the
 * session cookie on every server request, so server rendering and server
 * functions know who is signed in. With sign-in off it is left out entirely:
 * it would otherwise refuse to start without Clerk keys.
 */
import { clerkMiddleware } from "@clerk/tanstack-react-start/server";
import { createStart } from "@tanstack/react-start";

import { AUTH_MODE } from "#/lib/auth-mode";

export const startInstance = createStart(() => ({
	requestMiddleware: AUTH_MODE === "clerk" ? [clerkMiddleware()] : [],
}));
