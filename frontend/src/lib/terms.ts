/** How many chips one list may hold, matching the backend's column limit. */
export const MAX_TERMS = 10;

/**
 * Adds a typed term to a chip list the way the backend would store it:
 * trimmed, ignored when blank or already there in any casing, and ignored
 * once the list is full. Returns the list unchanged when nothing was added.
 */
export function addTerm(
	terms: readonly string[],
	draft: string,
	max = MAX_TERMS,
): string[] {
	const term = draft.trim();
	const alreadyThere = terms.some(
		(existing) => existing.toLowerCase() === term.toLowerCase(),
	);
	if (!term || alreadyThere || terms.length >= max) return [...terms];
	return [...terms, term];
}
