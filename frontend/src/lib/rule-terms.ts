/**
 * Which of a stored item's filter terms came from item rules rather than from
 * what the user sent. Compared case-insensitively, as the backend does when it
 * drops a rule term the user already typed.
 */
export function termsAddedByRules(
	sent: readonly string[] | undefined,
	stored: readonly string[],
): string[] {
	const typed = new Set((sent ?? []).map((term) => term.toLowerCase()));
	return stored.filter((term) => !typed.has(term.toLowerCase()));
}

/**
 * The toast description announcing rule filters, or `undefined` when no rule
 * added any — so an item that gained chips on its own is never a surprise.
 */
export function ruleTermsDescription(
	terms: readonly string[],
): string | undefined {
	if (terms.length === 0) return undefined;
	return `Item rules added the ${terms.length === 1 ? "filter" : "filters"}: ${terms.join(", ")}`;
}
