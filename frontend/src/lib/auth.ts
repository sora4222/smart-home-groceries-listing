/**
 * Auth token accessor — the one place the frontend reads a session token.
 *
 * TODO: wire up Clerk's `useAuth().getToken()` here once the Clerk
 * provider is installed (see docs/human-setup.md). Until then the backend
 * can be run with `DEV_AUTH_BYPASS=true` for local development, in which
 * case no token is required.
 */
export async function getAuthToken(): Promise<string | null> {
	return null;
}
