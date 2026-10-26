/** Reading Google's sign-in answer from the address it returns to. */
import type { GoogleReturn } from "#/hooks/useGoogleSignInReturn";

/** Keeps only Google's answer from the address, as strings. */
export function googleReturn(search: Record<string, unknown>): GoogleReturn {
	const found: GoogleReturn = {};
	for (const key of ["code", "state", "error"] as const) {
		const value = search[key];
		if (typeof value === "string" && value) found[key] = value;
	}
	return found;
}
