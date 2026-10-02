import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import { useCountRiseToast } from "#/hooks/useCountRiseToast";

/**
 * The spec's "Item held for review — tap to triage." toast, fired when triage
 * holds a new item. Mount once, at the root layout.
 */
export function useTriageToasts(): void {
	const navigate = useNavigate();
	useCountRiseToast("triage_held", () =>
		toast("Item held for review — tap to triage", {
			action: { label: "Triage", onClick: () => navigate({ to: "/triage" }) },
		}),
	);
}
