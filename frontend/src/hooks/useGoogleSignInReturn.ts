import { useNavigate, useRouter } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { toast } from "sonner";

import { api } from "#/lib/api";

/** What Google adds to `/settings/intake` when it sends the browser back. */
export interface GoogleReturn {
	code?: string;
	state?: string;
	error?: string;
}

/**
 * Finishes a Google sign-in when the page loads with Google's answer in the
 * address: hands the code and state to the backend once, then clears them
 * from the address bar (a code works only once) and reloads the settings.
 */
export function useGoogleSignInReturn({ code, state, error }: GoogleReturn) {
	const navigate = useNavigate();
	const router = useRouter();
	const handled = useRef(false);

	useEffect(() => {
		if (handled.current || (!error && !(code && state))) return;
		handled.current = true;

		async function finish() {
			if (error) {
				toast.error("Google sign-in was cancelled.");
			} else if (code && state) {
				try {
					await api.intake.finishSignIn(code, state);
					toast.success("Google Tasks connected. Now pick your grocery list.");
				} catch {
					toast.error("Could not connect Google Tasks — try again.");
				}
			}
			await navigate({ to: "/settings/intake", search: {}, replace: true });
			await router.invalidate();
		}
		void finish();
	}, [code, state, error, navigate, router]);
}
