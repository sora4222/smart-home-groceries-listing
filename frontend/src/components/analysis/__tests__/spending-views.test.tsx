import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { reportFixture } from "#/components/analysis/__tests__/fixtures";
import { SpendingViews } from "#/components/analysis/spending-views";
import type { AnalysisView } from "#/lib/analysis-search";
import { api } from "#/lib/api";

function renderViews(view: AnalysisView = "time") {
	const handlers = {
		onViewChange: vi.fn(),
		onPeriodChange: vi.fn(),
		onRecategorise: vi.fn(),
	};
	render(
		<SpendingViews
			report={reportFixture()}
			view={view}
			period="month"
			{...handlers}
		/>,
	);
	return handlers;
}

afterEach(() => vi.restoreAllMocks());

describe("SpendingViews", () => {
	it("shows the total with items and delivery apart", () => {
		renderViews();
		expect(screen.getByText(/^Spent:/)).toHaveTextContent(
			"Spent: $14.05 (items $12.05 + delivery $2.00)",
		);
	});

	it("draws each period over time and names the newest", () => {
		renderViews("time");
		expect(screen.getByRole("img", { name: /Spend per month/ })).toBeVisible();
		expect(screen.getByRole("status")).toHaveTextContent("Sep 2026: $3.10");
		expect(
			screen.getByRole("button", { name: "Aug 2026: $0.00" }),
		).toBeInTheDocument();
	});

	it("reads a point when it is focused", async () => {
		renderViews("time");
		screen.getByRole("button", { name: "Jul 2026: $10.95" }).focus();
		expect(await screen.findByRole("status")).toHaveTextContent(
			"Jul 2026: $10.95",
		);
	});

	it("asks for another grouping", async () => {
		const { onPeriodChange } = renderViews("time");
		await userEvent.click(screen.getByRole("button", { name: "Quarters" }));
		expect(onPeriodChange).toHaveBeenCalledWith("quarter");
	});

	it("asks for another view", async () => {
		const { onViewChange } = renderViews("time");
		await userEvent.click(screen.getByRole("tab", { name: "By store" }));
		expect(onViewChange).toHaveBeenCalledWith("store");
	});

	it("lists stores with items and delivery", () => {
		renderViews("store");
		const list = screen.getByRole("list", { name: "Spend by store" });
		expect(list).toHaveTextContent("Coles");
		expect(list).toHaveTextContent("items $4.95 + delivery $2.00");
		expect(list).toHaveTextContent("$6.95");
	});

	it("opens an item to the prices paid on every shop", async () => {
		const prices = vi.spyOn(api.spending, "itemPrices").mockResolvedValue([
			{
				bought_at: "2026-07-03T02:00:00Z",
				store: "coles",
				store_name: "Coles",
				product_name: "Full Cream Milk",
				brand: null,
				package_size: null,
				quantity: 1,
				unit_price: "4.95",
				total_price: "4.95",
			},
		]);
		renderViews("item");

		await userEvent.click(
			screen.getByRole("button", { name: "Show prices paid for milk" }),
		);

		const table = await screen.findByRole("table", {
			name: "Price paid for milk, oldest first",
		});
		expect(prices).toHaveBeenCalledWith("milk");
		expect(within(table).getByText("Coles · Full Cream Milk")).toBeVisible();
		expect(within(table).getByText("$4.95 × 1")).toBeVisible();
	});

	it("sorts categories again on request", async () => {
		const { onRecategorise } = renderViews("category");
		expect(
			screen.getByRole("list", { name: "Spend by category" }),
		).toHaveTextContent("Dairy & eggs");
		await userEvent.click(
			screen.getByRole("button", { name: "Sort categories again" }),
		);
		expect(onRecategorise).toHaveBeenCalled();
	});

	it("says when nothing was bought", () => {
		render(
			<SpendingViews
				report={reportFixture({ over_time: [], by_item: [] })}
				view="item"
				period="month"
				onViewChange={vi.fn()}
				onPeriodChange={vi.fn()}
				onRecategorise={vi.fn()}
			/>,
		);
		expect(screen.getByText("Nothing bought in this range.")).toBeVisible();
	});
});

describe("tickIndexes", () => {
	it("labels every point up to six, then five spread out", async () => {
		const { tickIndexes } = await import(
			"#/components/analysis/over-time-chart"
		);
		expect(tickIndexes(3)).toEqual([0, 1, 2]);
		expect(tickIndexes(6)).toEqual([0, 1, 2, 3, 4, 5]);
		expect(tickIndexes(13)).toEqual([0, 3, 6, 9, 12]);
	});
});
