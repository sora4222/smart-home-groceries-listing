import { createFileRoute, Outlet } from "@tanstack/react-router";

import { SectionLayout } from "#/components/section-layout";

export const Route = createFileRoute("/settings")({
	component: SettingsLayout,
});

/** `/settings` — every settings page, with its navigation on the left. */
function SettingsLayout() {
	return (
		<SectionLayout
			label="Settings sections"
			links={[
				{ to: "/settings/item-rules", label: "Item rules" },
				{ to: "/settings/dislikes", label: "Dislikes" },
				{ to: "/settings/delivery", label: "Delivery" },
				{ to: "/settings/display", label: "Display" },
				{ to: "/settings/logs", label: "Logs" },
				{ to: "/settings/help", label: "Help" },
			]}
		>
			<Outlet />
		</SectionLayout>
	);
}
