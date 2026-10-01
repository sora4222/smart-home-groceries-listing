import { Link } from "@tanstack/react-router";

import { Badge } from "#/components/ui/badge";
import { usePendingCount } from "#/hooks/usePendingCount";

const links: ReadonlyArray<{
	to: "/" | "/pending" | "/order" | "/settings/item-rules";
	label: string;
	badge?: true;
}> = [
	{ to: "/", label: "Grocery List" },
	{ to: "/pending", label: "Pending Requests", badge: true },
	{ to: "/order", label: "Order" },
	{ to: "/settings/item-rules", label: "Item Rules" },
];

/**
 * Top nav on desktop, collapses to a horizontally-scrollable bar on
 * narrow viewports (spec: mobile gets a bottom tab bar or hamburger —
 * a scrollable top bar is the simplest option that stays usable at
 * 390px without hidden functionality, and is swapped for a true bottom
 * tab bar as more routes land).
 */
export function Nav() {
	const pendingCount = usePendingCount();

	return (
		<nav
			aria-label="Main navigation"
			className="flex items-center gap-1 overflow-x-auto border-b border-border bg-background px-4 py-3"
		>
			{links.map((link) => (
				<Link
					key={link.to}
					to={link.to}
					className="flex shrink-0 items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground [&.active]:bg-accent [&.active]:text-accent-foreground"
				>
					{link.label}
					{link.badge && pendingCount > 0 && (
						<Badge
							variant="destructive"
							aria-label={`${pendingCount} pending requests`}
						>
							{pendingCount}
						</Badge>
					)}
				</Link>
			))}
		</nav>
	);
}
