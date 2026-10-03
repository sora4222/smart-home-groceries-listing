import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { itemFixture } from "#/components/grocery/__tests__/fixtures";
import { GroceryItemCard } from "#/components/grocery/grocery-item-card";
import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import type { GroceryItem } from "#/lib/api";

function renderCard(item: GroceryItem) {
	const onSave = vi.fn().mockResolvedValue(undefined);
	const onRemove = vi.fn().mockResolvedValue(undefined);
	render(<GroceryItemCard item={item} onSave={onSave} onRemove={onRemove} />);
	return { onSave, onRemove };
}

describe("GroceryItemCard", () => {
	it("shows the name and quantity", () => {
		renderCard(itemFixture());

		expect(screen.getByText("milk")).toBeInTheDocument();
		expect(screen.getByLabelText("Quantity 2")).toHaveTextContent("×2");
	});

	it("says where the item came from", () => {
		renderCard(itemFixture({ source: "voice" }));

		expect(screen.getByText("Added via voice")).toBeInTheDocument();
	});

	it("shows the note somebody left on it", () => {
		renderCard(itemFixture({ note: "the 2 litre bottle" }));

		expect(screen.getByText("the 2 litre bottle")).toBeInTheDocument();
	});

	it("shows the filter chips", () => {
		renderCard(itemFixture({ filter_terms: ["3 ply", "recycled"] }));

		const chips = screen.getByLabelText("Product filters");
		expect(chips).toHaveTextContent("3 ply");
		expect(chips).toHaveTextContent("recycled");
	});

	it("removes a chip from the list without opening the editor", async () => {
		const { onSave } = renderCard(
			itemFixture({ filter_terms: ["3 ply", "recycled"] }),
		);

		await userEvent.click(screen.getByLabelText("Remove filter 3 ply"));

		// The remaining chips are sent, which is how one removal is expressed.
		expect(onSave).toHaveBeenCalledWith(itemFixture().id, {
			filter_terms: ["recycled"],
		});
	});

	it("asks before removing an item", async () => {
		const { onRemove } = renderCard(itemFixture());

		await userEvent.click(screen.getByLabelText("Remove milk"));
		expect(onRemove).not.toHaveBeenCalled();

		await userEvent.click(screen.getByLabelText("Confirm removing milk"));
		expect(onRemove).toHaveBeenCalledWith(itemFixture().id);
	});

	it("lets the user back out of a removal", async () => {
		const { onRemove } = renderCard(itemFixture());

		await userEvent.click(screen.getByLabelText("Remove milk"));
		await userEvent.click(screen.getByRole("button", { name: "Keep" }));

		expect(onRemove).not.toHaveBeenCalled();
		expect(screen.getByLabelText("Edit milk")).toBeInTheDocument();
	});

	it("opens an editor for the whole item", async () => {
		renderCard(itemFixture({ note: "keep me", filter_terms: ["a2"] }));

		await userEvent.click(screen.getByLabelText("Edit milk"));

		expect(screen.getByLabelText("Item")).toHaveValue("milk");
		expect(screen.getByLabelText("Quantity")).toHaveValue(2);
		expect(screen.getByLabelText("Note")).toHaveValue("keep me");
	});

	it("is read-only once the list is committed", () => {
		renderCard(itemFixture({ status: "committed", filter_terms: ["3 ply"] }));

		expect(screen.getByText("Committed")).toBeInTheDocument();
		expect(screen.queryByLabelText("Edit milk")).not.toBeInTheDocument();
		expect(screen.queryByLabelText("Remove milk")).not.toBeInTheDocument();
		expect(
			screen.queryByLabelText("Remove filter 3 ply"),
		).not.toBeInTheDocument();
	});

	it("shows the product chosen for it and clears it", async () => {
		const onClearChoice = vi.fn().mockResolvedValue(undefined);
		render(
			<GroceryItemCard
				item={itemFixture()}
				onSave={vi.fn()}
				onRemove={vi.fn()}
				selection={selectionFixture()}
				onChoose={vi.fn()}
				onClearChoice={onClearChoice}
			/>,
		);

		expect(screen.getByLabelText("Chosen product for milk")).toHaveTextContent(
			"Coles Full Cream Milk",
		);
		await userEvent.click(
			screen.getByRole("button", { name: "Clear the chosen product for milk" }),
		);

		expect(onClearChoice).toHaveBeenCalledWith(itemFixture().id, "coles");
	});

	it("says when no product has been chosen", () => {
		renderCard(itemFixture());

		expect(screen.getByText("No product chosen yet")).toBeInTheDocument();
	});
});
