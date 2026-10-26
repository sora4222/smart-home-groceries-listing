import { Link, type LinkProps } from "@tanstack/react-router";
import type { ReactNode } from "react";

/** One link in a section's side navigation. */
export interface SectionLink {
	to: LinkProps["to"];
	label: string;
	/** Highlight only on this exact path (for a section's index page). */
	exact?: boolean;
}

/**
 * A page with its own navigation on the left (a row on top on phones),
 * used by Order and Settings.
 */
export function SectionLayout({
	label,
	links,
	children,
}: {
	/** The navigation's accessible name, e.g. "Order sections". */
	label: string;
	links: ReadonlyArray<SectionLink>;
	children: ReactNode;
}) {
	return (
		<div className="flex flex-col gap-4 md:flex-row md:items-start md:gap-6">
			<nav
				aria-label={label}
				className="flex shrink-0 gap-1 overflow-x-auto md:sticky md:top-4 md:w-44 md:flex-col"
			>
				{links.map((link) => (
					<Link
						key={String(link.to)}
						to={link.to}
						activeOptions={{ exact: link.exact ?? false }}
						className="shrink-0 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground [&.active]:bg-accent [&.active]:text-accent-foreground"
					>
						{link.label}
					</Link>
				))}
			</nav>
			<div className="min-w-0 flex-1">{children}</div>
		</div>
	);
}
