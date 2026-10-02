import { useLiveCount } from "#/hooks/useLiveCount";
import { api } from "#/lib/api";

const loadHeldCount = () =>
	api.triage.list("held").then((items) => items.length);

/**
 * The held-for-review count, for the Triage nav badge. Rejected requests are
 * not counted: the spec treats them as passive.
 */
export function useHeldCount(): number {
	return useLiveCount("triage_held", loadHeldCount);
}
