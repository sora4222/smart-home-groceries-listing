import { createFileRoute } from "@tanstack/react-router";

import { Badge } from "#/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { api } from "#/lib/api";

export const Route = createFileRoute("/")({
	loader: () => api.grocery.listActive(),
	component: Home,
});

/**
 * Active grocery list — read-only for now. Manual add/edit, item rules
 * and store selection land with the Grocery List and Order Optimisation
 * goals; this view exists so accepted voice requests have somewhere to
 * show up.
 */
function Home() {
	const items = Route.useLoaderData();

	if (items.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				Nothing on the list yet — accept a pending voice request or check back
				once manual add-item ships.
			</p>
		);
	}

	return (
		<div className="flex flex-col gap-3">
			<h1 className="text-lg font-semibold">Grocery List</h1>
			{items.map((item) => (
				<Card key={item.id}>
					<CardHeader className="flex-row items-center justify-between">
						<CardTitle>{item.name}</CardTitle>
						<Badge variant="secondary">×{item.quantity}</Badge>
					</CardHeader>
					<CardContent className="text-xs text-muted-foreground">
						Added via {item.source}
					</CardContent>
				</Card>
			))}
		</div>
	);
}
