import {
	createFileRoute,
	useNavigate,
	useRouter,
} from "@tanstack/react-router";
import { toast } from "sonner";

import { LatestOrderCard } from "#/components/analysis/latest-order-card";
import { RecentShops } from "#/components/analysis/recent-shops";
import { SpendingFilters } from "#/components/analysis/spending-filters";
import { SpendingViews } from "#/components/analysis/spending-views";
import { Button } from "#/components/ui/button";
import {
	type AnalysisSearch,
	filtersFor,
	validateAnalysisSearch,
} from "#/lib/analysis-search";
import { api } from "#/lib/api";

export const Route = createFileRoute("/analysis")({
	validateSearch: validateAnalysisSearch,
	loaderDeps: ({ search }) => search,
	loader: async ({ deps }) => {
		const [report, orders] = await Promise.all([
			api.spending.get(filtersFor(deps, new Date())),
			api.purchases.listOrders(),
		]);
		return { report, orders };
	},
	pendingComponent: () => (
		<p className="text-sm text-muted-foreground">Adding up your shops…</p>
	),
	errorComponent: AnalysisError,
	component: AnalysisPage,
});

/**
 * `/analysis` — Spending Analysis (spec "Spending Analysis").
 *
 * The newest shop on top, then the filters, then four views of the same
 * spending: over time, by item (with each item's price history), by store
 * and by category. Saved shops are listed last, each with Undo.
 */
function AnalysisPage() {
	const { report, orders } = Route.useLoaderData();
	const search = Route.useSearch();
	const navigate = useNavigate({ from: Route.fullPath });
	const router = useRouter();

	const update = (next: AnalysisSearch) =>
		navigate({ search: validateAnalysisSearch({ ...next }), replace: true });

	async function recategorise() {
		try {
			const { changed } = await api.purchases.recategorise();
			toast(
				changed === 0
					? "Categories are already up to date."
					: `${changed} ${changed === 1 ? "purchase" : "purchases"} moved to a new category.`,
			);
			await router.invalidate();
		} catch (err) {
			console.error("[analysis] could not sort categories", err);
			toast("Could not sort the categories just now.");
		}
	}

	// Bottom padding keeps the last Undo button clear of floating controls.
	return (
		<div className="flex flex-col gap-4 pb-16">
			<h1 className="text-lg font-semibold">Spending</h1>
			<LatestOrderCard order={report.latest_order} />
			<SpendingFilters
				search={search}
				categories={report.categories}
				onChange={update}
			/>
			<SpendingViews
				report={report}
				view={search.view ?? "time"}
				period={search.period ?? "month"}
				onViewChange={(view) => update({ ...search, view })}
				onPeriodChange={(period) => update({ ...search, period })}
				onRecategorise={recategorise}
			/>
			<RecentShops orders={orders} onChanged={() => router.invalidate()} />
		</div>
	);
}

/** The spending could not be read; offer to try again. */
function AnalysisError() {
	const router = useRouter();
	return (
		<div role="alert" className="flex flex-col items-start gap-3 text-sm">
			<p>Could not read your spending just now.</p>
			<Button variant="outline" onClick={() => router.invalidate()}>
				Try again
			</Button>
		</div>
	);
}
