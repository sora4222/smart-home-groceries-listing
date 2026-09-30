import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { TermListField } from "#/components/terms/term-list-field";

function renderField(terms: string[] = []) {
	const onChange = vi.fn();
	render(
		<TermListField
			id="triggers"
			label="Item names"
			noun="trigger"
			inputLabel="Add an item name"
			addLabel="Add name"
			maxLength={100}
			terms={terms}
			onChange={onChange}
		/>,
	);
	return { onChange };
}

describe("TermListField", () => {
	it("adds a typed term with the button", async () => {
		const { onChange } = renderField(["milk"]);

		await userEvent.type(screen.getByLabelText("Add an item name"), "eggs");
		await userEvent.click(screen.getByRole("button", { name: "Add name" }));

		expect(onChange).toHaveBeenCalledWith(["milk", "eggs"]);
	});

	it("adds on Enter", async () => {
		const { onChange } = renderField();

		await userEvent.type(
			screen.getByLabelText("Add an item name"),
			"eggs{Enter}",
		);

		expect(onChange).toHaveBeenCalledWith(["eggs"]);
	});

	it("names its chips' remove buttons after the noun", async () => {
		const { onChange } = renderField(["milk", "eggs"]);

		await userEvent.click(screen.getByLabelText("Remove trigger milk"));

		expect(onChange).toHaveBeenCalledWith(["eggs"]);
	});

	it("cannot add once full", async () => {
		renderField(Array.from({ length: 10 }, (_, n) => `item ${n}`));

		await userEvent.type(screen.getByLabelText("Add an item name"), "more");

		expect(screen.getByRole("button", { name: "Add name" })).toBeDisabled();
	});
});
