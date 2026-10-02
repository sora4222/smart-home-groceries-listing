/**
 * A thin strip saying sign-in is off, so nobody mistakes a test or local
 * setup for the real one. Renders nothing with sign-in on.
 */
import { AUTH_MODE } from "#/lib/auth-mode";

export function SignInOffNotice() {
	if (AUTH_MODE !== "off") return null;
	return (
		<output className="block border-b border-border bg-muted px-4 py-1 text-center text-xs text-muted-foreground">
			Sign-in is off. Use this only on your own computer.
		</output>
	);
}
