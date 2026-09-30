import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { AddItemForm } from "#/components/grocery/add-item-form";

describe("AddItemForm", () => {
	it("adds the typed item with a default quantity of one", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "milk");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(onAdd).toHaveBeenCalledWith({ name: "milk", quantity: 1 });
	});

	it("sends the quantity the user chose", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "bananas");
		await userEvent.clear(screen.getByLabelText("Quantity"));
		await userEvent.type(screen.getByLabelText("Quantity"), "3");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(onAdd).toHaveBeenCalledWith({ name: "bananas", quantity: 3 });
	});

	it("trims the name so a stray space does not become part of it", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "  oat milk  ");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(onAdd).toHaveBeenCalledWith({ name: "oat milk", quantity: 1 });
	});

	it("cannot be submitted empty", async () => {
		const onAdd = vi.fn();
		render(<AddItemForm onAdd={onAdd} />);

		expect(screen.getByRole("button", { name: "Add item" })).toBeDisabled();
		await userEvent.type(screen.getByLabelText("Item"), "   ");
		expect(screen.getByRole("button", { name: "Add item" })).toBeDisabled();
		expect(onAdd).not.toHaveBeenCalled();
	});

	it("reads an emptied quantity field as one rather than appending to it", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "rice");
		await userEvent.clear(screen.getByLabelText("Quantity"));
		await userEvent.type(screen.getByLabelText("Quantity"), "12");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(onAdd).toHaveBeenCalledWith({ name: "rice", quantity: 12 });
	});

	it("clamps a quantity above the column maximum", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "rice");
		await userEvent.clear(screen.getByLabelText("Quantity"));
		await userEvent.type(screen.getByLabelText("Quantity"), "5000");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(onAdd).toHaveBeenCalledWith({ name: "rice", quantity: 999 });
	});

	it("clears itself after a successful add, ready for the next item", async () => {
		const onAdd = vi.fn().mockResolvedValue(undefined);
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "bread");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(screen.getByLabelText("Item")).toHaveValue("");
		expect(screen.getByLabelText("Quantity")).toHaveValue(1);
	});

	it("keeps the draft when the add fails, so nothing is retyped", async () => {
		const onAdd = vi.fn().mockRejectedValue(new Error("offline"));
		render(<AddItemForm onAdd={onAdd} />);

		await userEvent.type(screen.getByLabelText("Item"), "rice");
		await userEvent.click(screen.getByRole("button", { name: "Add item" }));

		expect(screen.getByLabelText("Item")).toHaveValue("rice");
	});

	it("is closed while the list is committed", () => {
		render(<AddItemForm onAdd={vi.fn()} disabled />);

		expect(screen.getByLabelText("Item")).toBeDisabled();
		expect(screen.getByRole("button", { name: "Add item" })).toBeDisabled();
	});
});
