import { useLiveCount } from "#/hooks/useLiveCount";
import { api } from "#/lib/api";

const loadPendingCount = () =>
	api.voice.pending().then((items) => items.length);

/** The Pending Requests count, for the nav badge. */
export function usePendingCount(): number {
	return useLiveCount("voice_request_added", loadPendingCount);
}
