import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import { useCountRiseToast } from "#/hooks/useCountRiseToast";

/**
 * Fires the Sonner toast the spec calls for when a new item reaches Pending
 * Requests (after triage approved it). Mount once, at the root layout.
 */
export function useVoiceRequestToasts(): void {
	const navigate = useNavigate();
	useCountRiseToast("voice_request_added", () =>
		toast("New item added — tap to review", {
			action: { label: "Review", onClick: () => navigate({ to: "/pending" }) },
		}),
	);
}
