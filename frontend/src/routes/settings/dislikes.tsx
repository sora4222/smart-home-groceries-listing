import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { HouseholdDislikes } from "#/components/dislikes/household-dislikes";
import { api, type ProductChoice } from "#/lib/api";

export const Route = createFileRoute("/settings/dislikes")({
	loader: () => api.dislikes.list(),
	component: DislikesPage,
});

/**
 * Dislikes — every member's disliked products (spec: "Disliked items",
 * household visibility).
 *
 * A dislike is added from the price comparison. Here each member sees the
 * whole household's list and can remove their own. Mutations go through
 * `lib/api` and then `router.invalidate()`, so the loader stays the single
 * source of truth.
 */
function DislikesPage() {
	const dislikes = Route.useLoaderData();
	const router = useRouter();

	async function removeMine(choice: ProductChoice) {
		try {
			await api.dislikes.removeMine(choice);
			toast.success("Your dislike is removed");
			await router.invalidate();
		} catch {
			toast.error("Could not remove your dislike — try again.");
		}
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Dislikes</h1>
				<p className="text-sm text-muted-foreground">
					Products someone in the household does not want again. You can still
					choose them; they show a warning when you compare prices.
				</p>
			</div>
			<HouseholdDislikes dislikes={dislikes} onRemove={removeMine} />
		</div>
	);
}
