import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import { ChosenProduct } from "#/components/selections/chosen-product";

describe("ChosenProduct", () => {
	it("says when no product has been chosen yet", () => {
		render(<ChosenProduct itemName="milk" selection={null} />);

		expect(screen.getByText("No product chosen yet")).toBeInTheDocument();
		expect(screen.queryByRole("button")).not.toBeInTheDocument();
	});

	it("shows the chosen product, its size, store and price each", () => {
		render(
			<ChosenProduct
				itemName="milk"
				selection={selectionFixture()}
				onClear={vi.fn()}
			/>,
		);

		const chosen = screen.getByLabelText("Chosen product for milk");
		expect(chosen).toHaveTextContent("Coles Full Cream Milk");
		expect(chosen).toHaveTextContent("3L");
		expect(chosen).toHaveTextContent("Coles");
		expect(chosen).toHaveTextContent("$4.95 each");
	});

	it("clears the choice when asked", async () => {
		const onClear = vi.fn().mockResolvedValue(undefined);
		render(
			<ChosenProduct
				itemName="milk"
				selection={selectionFixture()}
				onClear={onClear}
			/>,
		);

		await userEvent.click(
			screen.getByRole("button", { name: "Clear the chosen product for milk" }),
		);

		expect(onClear).toHaveBeenCalledOnce();
	});

	it("offers no clear button when clearing is not allowed", () => {
		render(<ChosenProduct itemName="milk" selection={selectionFixture()} />);

		expect(screen.queryByRole("button")).not.toBeInTheDocument();
	});
});
