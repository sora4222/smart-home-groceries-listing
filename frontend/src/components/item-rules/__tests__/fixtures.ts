import type { ItemRule } from "#/lib/api";

/**
 * A rule with the fields a given test does not care about already filled in.
 */
export function ruleFixture(overrides: Partial<ItemRule> = {}): ItemRule {
	return {
		id: "44444444-4444-4444-4444-444444444444",
		triggers: ["toilet paper"],
		filter_terms: ["3 ply"],
		apply_to_manual: false,
		created_at: new Date("2026-09-30T00:00:00Z").toISOString(),
		updated_at: new Date("2026-09-30T00:00:00Z").toISOString(),
		...overrides,
	};
}
