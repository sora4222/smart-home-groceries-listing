import { TanStackDevtools } from "@tanstack/react-devtools";
import {
	createRootRoute,
	HeadContent,
	Outlet,
	Scripts,
} from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import type { ReactNode } from "react";
import { Toaster } from "sonner";

import { Nav } from "#/components/nav";
import { useDesktopNotifications } from "#/hooks/useDesktopNotifications";
import { useHydrated } from "#/hooks/useHydrated";
import { useTriageToasts } from "#/hooks/useTriageToasts";
import { useVoiceRequestToasts } from "#/hooks/useVoiceRequestToasts";

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
	component: RootComponent,
});

function RootComponent() {
	return (
		<RootDocument>
			<Nav />
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
	useDesktopNotifications();
	const hydrated = useHydrated();

	return (
		<html lang="en">
			<head>
				<HeadContent />
			</head>
			<body>
				<div id="app" data-hydrated={hydrated}>
					{children}
				</div>
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
