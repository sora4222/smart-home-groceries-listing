import { TanStackDevtools } from "@tanstack/react-devtools";
import {
	createRootRoute,
	HeadContent,
	Outlet,
	redirect,
	Scripts,
	useLocation,
} from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import type { ReactNode } from "react";
import { Toaster } from "sonner";

import { AuthProvider } from "#/components/auth/auth-provider";
import { SignInOffNotice } from "#/components/auth/sign-in-off-notice";
import { Nav } from "#/components/nav";
import { useHydrated } from "#/hooks/useHydrated";
import { useTriageToasts } from "#/hooks/useTriageToasts";
import { useVoiceRequestToasts } from "#/hooks/useVoiceRequestToasts";
import { AUTH_MODE } from "#/lib/auth-mode";
import { fetchSignedIn } from "#/lib/auth-state";
import {
	guardDecision,
	isAuthPath,
	needsSessionCheck,
	SIGN_IN_PATH,
} from "#/lib/sign-in-guard";

import appCss from "#/styles.css?url";

export const Route = createRootRoute({
	head: () => ({
		meta: [
			{ charSet: "utf-8" },
			{ name: "viewport", content: "width=device-width, initial-scale=1" },
			{ title: "Grocery List" },
		],
		links: [
			{ rel: "stylesheet", href: appCss },
			// Without an explicit icon every page load asks for /favicon.ico and
			// logs a 404 in the console.
			{ rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
		],
	}),
	beforeLoad: requireSignIn,
	component: RootComponent,
});

/**
 * Sends a signed-out visitor to `/sign-in` before any page loader runs, so
 * no page asks the backend for data it would refuse. Does nothing with
 * sign-in off.
 */
async function requireSignIn({ location }: { location: { pathname: string } }) {
	if (!needsSessionCheck(AUTH_MODE, location.pathname)) return;
	const signedIn = await fetchSignedIn();
	if (guardDecision(AUTH_MODE, location.pathname, signedIn) === "sign-in") {
		throw redirect({ href: SIGN_IN_PATH });
	}
}

function RootComponent() {
	const { pathname } = useLocation();
	return (
		<RootDocument>
			<SignInOffNotice />
			{!isAuthPath(pathname) && <Nav />}
			<main className="mx-auto max-w-3xl p-4">
				<Outlet />
			</main>
			<Toaster richColors position="top-right" />
		</RootDocument>
	);
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
	useVoiceRequestToasts();
	useTriageToasts();
	const hydrated = useHydrated();

	return (
		<html lang="en">
			<head>
				<HeadContent />
			</head>
			<body>
				<AuthProvider>
					<div id="app" data-hydrated={hydrated}>
						{children}
					</div>
				</AuthProvider>
				<TanStackDevtools
					config={{ position: "bottom-right" }}
					plugins={[
						{
							name: "TanStack Router",
							render: <TanStackRouterDevtoolsPanel />,
						},
					]}
				/>
				<Scripts />
			</body>
		</html>
	);
}
