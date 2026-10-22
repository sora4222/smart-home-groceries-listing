/**
 * The desktop notification for a finished trolley fill: how many products
 * went in, how many did not, and what to do next.
 */
import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";

/** A notification's title and body. */
export interface Notice {
	title: string;
	body: string;
}

/** The notice for `handoff`, or `null` while it is not filled yet. */
export function handoffNotice(handoff: TrolleyHandoff): Notice | null {
	if (handoff.status !== "filled" && handoff.status !== "filled_with_problems")
		return null;
	const added = handoff.lines.filter((line) => line.outcome === "added").length;
	const failed = handoff.lines.filter(
		(line) => line.outcome === "failed",
	).length;
	const parts = [`${added} added.`];
	if (failed > 0) parts.push(`${failed} not added.`);
	parts.push("Check out when ready.");
	return {
		title: `${handoff.store_name} trolley filled`,
		body: parts.join(" "),
	};
}
