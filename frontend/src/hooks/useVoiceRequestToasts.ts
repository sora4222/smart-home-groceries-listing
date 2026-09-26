import { useEffect, useRef } from "react";
import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import { connect, subscribe } from "#/lib/ws";

/**
 * Fires the Sonner toast the spec calls for when a *new* voice request
 * arrives (not on every count change from an accept/reject decision).
 * Mount once, at the root layout.
 */
export function useVoiceRequestToasts(): void {
	const navigate = useNavigate();
	const previousCount = useRef<number | null>(null);

	useEffect(() => {
		connect();
		return subscribe((event) => {
			if (event.type !== "voice_request_added") return;
			if (
				previousCount.current !== null &&
				event.count > previousCount.current
			) {
				toast("New item added via Google Home — tap to review", {
					action: {
						label: "Review",
						onClick: () => navigate({ to: "/pending" }),
					},
				});
			}
			previousCount.current = event.count;
		});
	}, [navigate]);
}
