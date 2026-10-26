import { createFileRoute, Outlet } from "@tanstack/react-router";

import { SectionLayout } from "#/components/section-layout";

export const Route = createFileRoute("/order")({
	component: OrderLayout,
});

/** `/order` — In Progress (the order being put together) and History. */
function OrderLayout() {
	return (
		<SectionLayout
			label="Order sections"
			links={[
				{ to: "/order", label: "In Progress", exact: true },
				{ to: "/order/history", label: "History" },
			]}
		>
			<Outlet />
		</SectionLayout>
	);
}
