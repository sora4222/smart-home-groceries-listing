import { createFileRoute } from "@tanstack/react-router";

/** `/settings` before a section is picked. */
export const Route = createFileRoute("/settings/")({
	component: () => (
		<p className="text-sm text-muted-foreground">Pick a section.</p>
	),
});
