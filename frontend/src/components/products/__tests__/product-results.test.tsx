import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { itemFixture } from "#/components/grocery/__tests__/fixtures";
import {
	productFixture,
	searchFixture,
	storeFixture,
} from "#/components/products/__tests__/fixtures";
import { ProductChoiceProvider } from "#/components/products/product-choice";
import { ProductResults } from "#/components/products/product-results";
import { api, type ItemProducts } from "#/lib/api";
import { DEFAULT_DISPLAY, saveDisplay } from "#/lib/display-settings";

function renderResults(search: ItemProducts) {
	vi.spyOn(api.products, "forItem").mockResolvedValue(search);
	render(<ProductResults item={itemFixture()} />);
	return screen.findByRole("list", { name: "Products" });
}

/** The product names in a list, top to bottom. */
function articleNames(list: HTMLElement) {
	return within(list)
		.getAllByRole("article")
		.map((article) => article.getAttribute("aria-label"));
}

async function pick(label: string, option: string) {
	await userEvent.click(screen.getByRole("combobox", { name: label }));
	await userEvent.click(await screen.findByRole("option", { name: option }));
}

const bothStores = searchFixture([
	storeFixture({
		store: "woolworths",
		store_name: "Woolworths",
		products: [productFixture({ product_id: "w", brand: "B", price: "2.00" })],
	}),
	storeFixture({
		products: [
			productFixture({ product_id: "c1", brand: "C", price: "1.00" }),
			productFixture({ product_id: "c2", brand: "A", price: "3.00" }),
		],
	}),
]);

afterEach(() => {
	vi.restoreAllMocks();
	window.localStorage.clear();
});

describe("ProductResults", () => {
	it("searches for the item and shows both stores in one list", async () => {
		const list = await renderResults(bothStores);

		expect(api.products.forItem).toHaveBeenCalledWith(
			itemFixture().id,
			expect.anything(),
		);
		expect(articleNames(list)).toHaveLength(3);
	});

	it("sorts both stores together", async () => {
		const list = await renderResults(bothStores);

		await pick("Sort by", "Price: low to high");
		expect(articleNames(list)).toEqual([
			"C Full Cream Milk",
			"B Full Cream Milk",
			"A Full Cream Milk",
		]);

		await pick("Sort by", "Name: A to Z");
		expect(articleNames(list)).toEqual([
			"A Full Cream Milk",
			"B Full Cream Milk",
			"C Full Cream Milk",
		]);
	});

	it("filters by store and by price", async () => {
		const list = await renderResults(bothStores);

		await pick("Store", "Coles");
		expect(articleNames(list).sort()).toEqual([
			"A Full Cream Milk",
			"C Full Cream Milk",
		]);

		await userEvent.type(screen.getByLabelText("Max price ($)"), "2");
		expect(articleNames(list)).toEqual(["C Full Cream Milk"]);
	});

	it("filters by grams or litres", async () => {
		await renderResults(
			searchFixture([
				storeFixture({
					products: [
						productFixture({ product_id: "milk", name: "Milk" }),
						productFixture({
							product_id: "rice",
							name: "Rice",
							unit_price: { amount: "0.5", per: "100g", converted_from: null },
						}),
					],
				}),
			]),
		);

		await pick("Size", "Grams");
		expect(screen.queryByRole("article", { name: "Coles Milk" })).toBeNull();
		expect(screen.getByRole("article", { name: "Coles Rice" })).toBeVisible();
	});

	it("narrows to specials when asked", async () => {
		await renderResults(
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

		await userEvent.click(
			screen.getByRole("switch", { name: "Specials only" }),
		);

		expect(screen.queryByRole("article", { name: "Coles Plain" })).toBeNull();
		expect(screen.getByRole("article", { name: "Coles Cheap" })).toBeVisible();
	});

	it("hides unavailable products when the display setting says so", async () => {
		saveDisplay({ ...DEFAULT_DISPLAY, showUnavailable: false });
		const list = await renderResults(
			searchFixture([
				storeFixture({
					products: [
						productFixture({ product_id: "a", name: "Here" }),
						productFixture({ product_id: "b", name: "Gone", available: false }),
					],
				}),
			]),
		);

		expect(articleNames(list)).toEqual(["Coles Here"]);
	});

	it("offers to try again when the search fails", async () => {
		const search = vi
			.spyOn(api.products, "forItem")
			.mockRejectedValueOnce(new Error("offline"))
			.mockResolvedValue(searchFixture([storeFixture()]));
		render(<ProductResults item={itemFixture()} />);

		expect(await screen.findByRole("alert")).toHaveTextContent(
			"The stores could not be searched.",
		);
		await userEvent.click(screen.getByRole("button", { name: "Try again" }));

		expect(
			await screen.findByRole("list", { name: "Products" }),
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
			<ProductChoiceProvider
				value={{ chosen: [{ store: "coles", product_id: "a" }], onChoose }}
			>
				<ProductResults item={itemFixture()} />
			</ProductChoiceProvider>,
		);
		await screen.findByRole("list", { name: "Products" });

		expect(
			screen.getByRole("button", { name: "Coles Plain at Coles is chosen" }),
		).toHaveAttribute("aria-pressed", "true");
		await userEvent.click(
			screen.getByRole("button", { name: "Choose Coles Cheap at Coles" }),
		);

		expect(onChoose).toHaveBeenCalledWith({ store: "coles", product_id: "b" });
	});

	it("only compares when nothing can be chosen", async () => {
		await renderResults(searchFixture([storeFixture()]));

		expect(screen.queryByRole("button", { name: /^Choose / })).toBeNull();
	});
});
