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

		expect(onClear).toHaveBeenCalledWith("coles");
	});

	it("shows the item's product at the other store beside the one to buy", async () => {
		const onClear = vi.fn().mockResolvedValue(undefined);
		const atWoolworths = selectionFixture({
			store: "woolworths",
			store_name: "Woolworths",
			product_id: "w-milk-2l",
			brand: "Woolworths",
			package_size: "2L",
			price: "3.10",
		});
		render(
			<ChosenProduct
				itemName="milk"
				selection={selectionFixture({ also_chosen: [atWoolworths] })}
				onClear={onClear}
			/>,
		);

		const other = screen.getByLabelText("Also chosen at Woolworths for milk");
		expect(other).toHaveTextContent("Also at Woolworths");
		expect(other).toHaveTextContent("Woolworths Full Cream Milk");
		expect(other).toHaveTextContent("$3.10 each");

		await userEvent.click(
			screen.getByRole("button", {
				name: "Clear the Woolworths product for milk",
			}),
		);
		expect(onClear).toHaveBeenCalledWith("woolworths");
	});

	it("offers no clear button when clearing is not allowed", () => {
		render(<ChosenProduct itemName="milk" selection={selectionFixture()} />);

		expect(screen.queryByRole("button")).not.toBeInTheDocument();
	});
});
