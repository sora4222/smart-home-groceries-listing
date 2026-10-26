import { Link } from "@tanstack/react-router";

import { AccountButton } from "#/components/auth/account-button";
import { Badge } from "#/components/ui/badge";
import { useHeldCount } from "#/hooks/useHeldCount";
import { usePendingCount } from "#/hooks/usePendingCount";

const links: ReadonlyArray<{
	to:
		| "/"
		| "/pending"
		| "/order"
		| "/analysis"
		| "/settings/item-rules"
		| "/settings/intake"
		| "/settings/dislikes"
		| "/settings/delivery"
		| "/logs";
	label: string;
	/** Shows how many requests wait on the Pending page. */
	waiting?: boolean;
}> = [
	{ to: "/", label: "Grocery List" },
	{ to: "/pending", label: "Pending", waiting: true },
	{ to: "/order", label: "Order" },
	{ to: "/analysis", label: "Spending" },
	{ to: "/settings/item-rules", label: "Item Rules" },
	{ to: "/settings/intake", label: "Intake" },
	{ to: "/settings/dislikes", label: "Dislikes" },
	{ to: "/settings/delivery", label: "Delivery" },
	{ to: "/logs", label: "Logs" },
];

/**
 * Top nav on desktop, collapses to a horizontally-scrollable bar on
 * narrow viewports (spec: mobile gets a bottom tab bar or hamburger —
 * a scrollable top bar is the simplest option that stays usable at
 * 390px without hidden functionality, and is swapped for a true bottom
 * tab bar as more routes land).
 */
export function Nav() {
	// Pending shows both requests to accept and items held for review.
	const waiting = usePendingCount() + useHeldCount();

	return (
		<nav
			aria-label="Main navigation"
			className="flex items-center gap-1 overflow-x-auto border-b border-border bg-background px-4 py-3"
		>
			{links.map((link) => {
				const count = link.waiting ? waiting : 0;
				return (
					<Link
						key={link.to}
						to={link.to}
						activeOptions={{ exact: link.to === "/" }}
						className="flex shrink-0 items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground [&.active]:bg-accent [&.active]:text-accent-foreground"
					>
						{link.label}
						{count > 0 && (
							<Badge variant="destructive" aria-label={`${count} waiting`}>
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
