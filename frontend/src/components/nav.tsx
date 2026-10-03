import { Link } from "@tanstack/react-router";

import { AccountButton } from "#/components/auth/account-button";
import { Badge } from "#/components/ui/badge";
import { useHeldCount } from "#/hooks/useHeldCount";
import { usePendingCount } from "#/hooks/usePendingCount";

/** Which live count a link's badge shows, if any. */
type BadgeKind = "pending" | "held";

const links: ReadonlyArray<{
	to:
		| "/"
		| "/pending"
		| "/triage"
		| "/order"
		| "/analysis"
		| "/settings/item-rules";
	label: string;
	badge?: BadgeKind;
}> = [
	{ to: "/", label: "Grocery List" },
	{ to: "/pending", label: "Pending Requests", badge: "pending" },
	{ to: "/triage", label: "Triage", badge: "held" },
	{ to: "/order", label: "Order" },
	{ to: "/analysis", label: "Spending" },
	{ to: "/settings/item-rules", label: "Item Rules" },
];

/** What a badge's count is read aloud as. */
const badgeLabels: Record<BadgeKind, string> = {
	pending: "pending requests",
	held: "held for review",
};

/**
 * Top nav on desktop, collapses to a horizontally-scrollable bar on
 * narrow viewports (spec: mobile gets a bottom tab bar or hamburger —
 * a scrollable top bar is the simplest option that stays usable at
 * 390px without hidden functionality, and is swapped for a true bottom
 * tab bar as more routes land).
 */
export function Nav() {
	const counts: Record<BadgeKind, number> = {
		pending: usePendingCount(),
		held: useHeldCount(),
	};

	return (
		<nav
			aria-label="Main navigation"
			className="flex items-center gap-1 overflow-x-auto border-b border-border bg-background px-4 py-3"
		>
			{links.map((link) => {
				const count = link.badge ? counts[link.badge] : 0;
				return (
					<Link
						key={link.to}
						to={link.to}
						className="flex shrink-0 items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground [&.active]:bg-accent [&.active]:text-accent-foreground"
					>
						{link.label}
						{link.badge && count > 0 && (
							<Badge
								variant="destructive"
								aria-label={`${count} ${badgeLabels[link.badge]}`}
							>
								{count}
							</Badge>
						)}
					</Link>
				);
			})}
			<AccountButton />
		</nav>
	);
}
