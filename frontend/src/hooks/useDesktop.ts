import { useEffect, useState } from "react";

import { isDesktop } from "#/lib/desktop/bridge";
import { type DesktopAbilities, desktop } from "#/lib/desktop/commands";

/**
 * What the desktop app can do, or `null` in a browser (and until the app
 * has answered). Read after the page has hydrated, so server rendering and
 * the first client render always match.
 */
export function useDesktop(): DesktopAbilities | null {
	const [abilities, setAbilities] = useState<DesktopAbilities | null>(null);
	useEffect(() => {
		if (!isDesktop()) return;
		let cancelled = false;
		desktop
			.abilities()
			.then((found) => {
				if (!cancelled) setAbilities(found);
			})
			.catch((err) => console.error("[desktop] no abilities", err));
		return () => {
			cancelled = true;
		};
	}, []);
	return abilities;
}
