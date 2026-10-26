import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { itemFixture } from "#/components/grocery/__tests__/fixtures";
import {
	productFixture,
	searchFixture,
	storeFixture,
} from "#/components/products/__tests__/fixtures";
import { PriceComparison } from "#/components/products/price-comparison";
import { api } from "#/lib/api";

function renderComparison() {
	render(
		<PriceComparison item={itemFixture({ filter_terms: ["full cream"] })}>
			<PriceComparison.Trigger />
			<PriceComparison.Content />
		</PriceComparison>,
	);
}

/** The product names in a list, top to bottom. */
function articleNames(list: HTMLElement) {
	return within(list)
		.getAllByRole("article")
		.map((article) => article.getAttribute("aria-label"));
}

async function pickSort(dialog: HTMLElement, label: string) {
	await userEvent.click(
		within(dialog).getByRole("combobox", { name: "Sort by" }),
	);
	await userEvent.click(await screen.findByRole("option", { name: label }));
}

async function open() {
	await userEvent.click(
		screen.getByRole("button", { name: "Compare prices for milk" }),
	);
	return screen.findByRole("dialog");
}

afterEach(() => {
	vi.restoreAllMocks();
});

describe("PriceComparison", () => {
	it("does not search the stores until it is opened", () => {
		const search = vi.spyOn(api.products, "forItem");
		renderComparison();

		expect(search).not.toHaveBeenCalled();
	});

	it("shows both stores' products in one list once opened", async () => {
		const search = vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([
				storeFixture({
					store: "woolworths",
					store_name: "Woolworths",
					products: [productFixture({ product_id: "w1", brand: "Woolworths" })],
				}),
				storeFixture(),
			]),
		);
		renderComparison();

		const dialog = await open();

		expect(search).toHaveBeenCalledWith(
			"33333333-3333-3333-3333-333333333333",
			expect.anything(),
		);
		expect(
			within(dialog).getByRole("heading", { name: "Prices for milk ×2" }),
		).toBeInTheDocument();
		expect(within(dialog).getByLabelText("Product filters")).toHaveTextContent(
			"full cream",
		);
		const list = await within(dialog).findByRole("list", { name: "Products" });
		expect(articleNames(list)).toEqual([
			"Woolworths Full Cream Milk",
			"Coles Full Cream Milk",
		]);
	});

	it("sorts both stores together by what the household picks", async () => {
		vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([
				storeFixture({
					store: "woolworths",
					store_name: "Woolworths",
					products: [
						productFixture({ product_id: "w", brand: "B", price: "2.00" }),
					],
				}),
				storeFixture({
					products: [
						productFixture({ product_id: "c1", brand: "C", price: "1.00" }),
						productFixture({ product_id: "c2", brand: "A", price: "3.00" }),
					],
				}),
			]),
		);
		renderComparison();
		const dialog = await open();
		const list = await within(dialog).findByRole("list", { name: "Products" });

		await pickSort(dialog, "Price: low to high");
		expect(articleNames(list)).toEqual([
			"C Full Cream Milk",
			"B Full Cream Milk",
			"A Full Cream Milk",
		]);

		await pickSort(dialog, "Name: A to Z");
		expect(articleNames(list)).toEqual([
			"A Full Cream Milk",
			"B Full Cream Milk",
			"C Full Cream Milk",
		]);
	});

	it("narrows to specials when asked", async () => {
		vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([
				storeFixture({
					products: [
						productFixture({ product_id: "a", name: "Plain" }),
						productFixture({
							product_id: "b",
							name: "Cheap",
							on_special: true,
						}),
					],
				}),
			]),
		);
		renderComparison();
		const dialog = await open();
		await within(dialog).findByRole("list", { name: "Products" });

		await userEvent.click(
			within(dialog).getByRole("switch", { name: "Specials only" }),
		);

		expect(
			within(dialog).queryByRole("article", { name: "Coles Plain" }),
		).toBeNull();
		expect(
			within(dialog).getByRole("article", { name: "Coles Cheap" }),
		).toBeInTheDocument();
	});

	it("offers to try again when the search fails", async () => {
		const search = vi
			.spyOn(api.products, "forItem")
			.mockRejectedValueOnce(new Error("offline"))
			.mockResolvedValue(searchFixture([storeFixture()]));
		renderComparison();
		const dialog = await open();

		expect(await within(dialog).findByRole("alert")).toHaveTextContent(
			"The stores could not be searched.",
		);
		await userEvent.click(
			within(dialog).getByRole("button", { name: "Try again" }),
		);

		expect(
			await within(dialog).findByRole("list", { name: "Products" }),
		).toBeInTheDocument();
		expect(search).toHaveBeenCalledTimes(2);
	});

	it("chooses a product and shows which one is chosen", async () => {
		vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([
				storeFixture({
					products: [
						productFixture({ product_id: "a", name: "Plain" }),
						productFixture({ product_id: "b", name: "Cheap" }),
					],
				}),
			]),
		);
		const onChoose = vi.fn().mockResolvedValue(undefined);
		render(
			<PriceComparison
				item={itemFixture()}
				chosen={[{ store: "coles", product_id: "a" }]}
				onChoose={onChoose}
			>
				<PriceComparison.Trigger />
				<PriceComparison.Content />
			</PriceComparison>,
		);
		const dialog = await open();
		await within(dialog).findByRole("list", { name: "Products" });

		expect(
			within(dialog).getByRole("button", {
				name: "Coles Plain at Coles is chosen",
			}),
		).toHaveAttribute("aria-pressed", "true");
		await userEvent.click(
			within(dialog).getByRole("button", {
				name: "Choose Coles Cheap at Coles",
			}),
		);

		expect(onChoose).toHaveBeenCalledWith({ store: "coles", product_id: "b" });
	});

	it("only compares when nothing can be chosen", async () => {
		vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([storeFixture()]),
		);
		renderComparison();
		const dialog = await open();
		await within(dialog).findByRole("list", { name: "Products" });

		expect(
			within(dialog).queryByRole("button", { name: /^Choose / }),
		).toBeNull();
	});
});
