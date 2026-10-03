/**
 * Whether the person viewing the page is signed in, asked of the server so
 * the answer is the same during server rendering and in the browser.
 *
 * Used by the root route's `beforeLoad` to send signed-out visitors to
 * `/sign-in`. Only called with sign-in on.
 */
import { createServerFn } from "@tanstack/react-start";

/** `true` when the request carries a valid Clerk session. */
export const fetchSignedIn = createServerFn({ method: "GET" }).handler(
	async (): Promise<boolean> => {
		const { auth } = await import("@clerk/tanstack-react-start/server");
		return (await auth()).isAuthenticated;
	},
);
