import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { itemFixture } from "#/components/grocery/__tests__/fixtures";
import { GroceryItemEditor } from "#/components/grocery/grocery-item-editor";
import type { GroceryItem } from "#/lib/api";

function renderEditor(item: GroceryItem = itemFixture()) {
	const onSave = vi.fn().mockResolvedValue(undefined);
	const onCancel = vi.fn();
	render(<GroceryItemEditor item={item} onSave={onSave} onCancel={onCancel} />);
	return { onSave, onCancel };
}

describe("GroceryItemEditor", () => {
	it("saves a renamed, re-quantified, annotated item", async () => {
		const { onSave } = renderEditor();

		await userEvent.clear(screen.getByLabelText("Item"));
		await userEvent.type(screen.getByLabelText("Item"), "full cream milk");
		await userEvent.clear(screen.getByLabelText("Quantity"));
		await userEvent.type(screen.getByLabelText("Quantity"), "4");
		await userEvent.type(screen.getByLabelText("Note"), "the 2 litre bottle");
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith({
			name: "full cream milk",
			quantity: 4,
			note: "the 2 litre bottle",
			filter_terms: [],
		});
	});

	it("sends an empty note to clear an annotation", async () => {
		const { onSave } = renderEditor(itemFixture({ note: "remove me" }));

		await userEvent.clear(screen.getByLabelText("Note"));
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ note: "" }));
	});

	it("adds a filter chip", async () => {
		const { onSave } = renderEditor();

		await userEvent.type(
			screen.getByLabelText("Add a product filter"),
			"3 ply",
		);
		await userEvent.click(screen.getByRole("button", { name: "Add filter" }));
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({ filter_terms: ["3 ply"] }),
		);
	});

	it("adds a filter chip on Enter without submitting the page", async () => {
		renderEditor();

		await userEvent.type(
			screen.getByLabelText("Add a product filter"),
			"organic{Enter}",
		);

		expect(screen.getByLabelText("Remove filter organic")).toBeInTheDocument();
	});

	it("ignores a repeated chip whatever its casing", async () => {
		const { onSave } = renderEditor(itemFixture({ filter_terms: ["3 ply"] }));

		await userEvent.type(
			screen.getByLabelText("Add a product filter"),
			"3 PLY{Enter}",
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({ filter_terms: ["3 ply"] }),
		);
	});

	it("drops a chip the user takes off", async () => {
		const { onSave } = renderEditor(
			itemFixture({ filter_terms: ["3 ply", "recycled"] }),
		);

		await userEvent.click(screen.getByLabelText("Remove filter recycled"));
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({ filter_terms: ["3 ply"] }),
		);
	});

	it("will not save an item with no name", async () => {
		const { onSave } = renderEditor();

		await userEvent.clear(screen.getByLabelText("Item"));

		expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
		expect(onSave).not.toHaveBeenCalled();
	});

	it("leaves the item alone when cancelled", async () => {
		const { onSave, onCancel } = renderEditor();

		await userEvent.type(screen.getByLabelText("Item"), " oat");
		await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

		expect(onCancel).toHaveBeenCalledOnce();
		expect(onSave).not.toHaveBeenCalled();
	});
});
