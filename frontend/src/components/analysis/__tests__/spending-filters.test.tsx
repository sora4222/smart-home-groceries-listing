import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { SpendingFilters } from "#/components/analysis/spending-filters";
import type { AnalysisSearch } from "#/lib/analysis-search";

function renderFilters(search: AnalysisSearch = {}) {
	const onChange = vi.fn();
	render(
		<SpendingFilters
			search={search}
			categories={["Bakery", "Dairy & eggs"]}
			onChange={onChange}
		/>,
	);
	return onChange;
}

describe("SpendingFilters", () => {
	it("changes the date range", async () => {
		const onChange = renderFilters({ store: "coles" });
		await userEvent.selectOptions(screen.getByLabelText("Dates"), "month");
		expect(onChange).toHaveBeenCalledWith({ store: "coles", range: "month" });
	});

	it("shows day pickers only for chosen days", () => {
		renderFilters({ range: "custom", from: "2026-09-01" });
		expect(screen.getByLabelText("From")).toHaveValue("2026-09-01");
		expect(screen.getByLabelText("To")).toHaveValue("");
	});

	it("filters by store and category, and clears them", async () => {
		const onChange = renderFilters({ category: "Bakery" });
		await userEvent.selectOptions(screen.getByLabelText("Store"), "woolworths");
		expect(onChange).toHaveBeenLastCalledWith({
			category: "Bakery",
			store: "woolworths",
		});
		await userEvent.selectOptions(
			screen.getByLabelText("Category"),
			"All categories",
		);
		expect(onChange).toHaveBeenLastCalledWith({ category: undefined });
	});

	it("searches item names on Find", async () => {
		const onChange = renderFilters();
		await userEvent.type(screen.getByLabelText("Item name"), "  milk ");
		expect(onChange).not.toHaveBeenCalled();
		await userEvent.click(screen.getByRole("button", { name: "Find" }));
		expect(onChange).toHaveBeenCalledWith({ item: "milk" });
	});
});
