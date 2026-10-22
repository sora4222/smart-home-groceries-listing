import { useEffect, useRef } from "react";

import { useFillTrolleyConfig } from "#/hooks/useFillTrolleyConfig";
import type { StoreTab } from "#/lib/store-tab/store-tab";

/**
 * A store's "Fill … trolley" link, to drag to the bookmarks bar once.
 *
 * It is built with the store-tab secret from the backend. React refuses
 * `javascript:` URLs in `href`, so the link's address is set on the element
 * directly after it renders.
 */
export function FillTrolleyBookmark({ tab }: { tab: StoreTab }) {
	const link = useRef<HTMLAnchorElement>(null);
	const { config, failed } = useFillTrolleyConfig();
	const href = config ? tab.buildBookmarklet(config) : null;

	useEffect(() => {
		if (href) link.current?.setAttribute("href", href);
	}, [href]);

	if (failed) {
		return (
			<p className="text-sm text-destructive">
				The bookmark cannot be made: STORE_TAB_SECRET is not set on the server.
			</p>
		);
	}
	return (
		// biome-ignore lint/a11y/useValidAnchor: the javascript: href is set through the ref above, because React refuses it as a prop.
		<a
			ref={link}
			aria-disabled={!href}
			className="inline-flex w-fit cursor-grab rounded-md border px-3 py-2 text-sm font-medium"
		>
			{href ? tab.bookmarkName : "Making the bookmark…"}
		</a>
	);
}
