import { useEffect, useState } from "react";

/**
 * False during server rendering and the first client render, true once React
 * has hydrated the page.
 *
 * The app is server-rendered, so markup is interactive-looking before React has
 * attached a single handler: a click or a keystroke in that window is silently
 * lost. The root document publishes this as `#app[data-hydrated]` so the e2e
 * tests can wait for the page to actually be live, and so a "why did my click
 * do nothing" moment is visible in dev tools.
 */
export function useHydrated(): boolean {
	const [hydrated, setHydrated] = useState(false);
	useEffect(() => setHydrated(true), []);
	return hydrated;
}
