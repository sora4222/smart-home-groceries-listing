import { addItem, expect, gotoList, itemCard, test, openSelectItem, backToList } from "./fixtures";
import type { Page } from "@playwright/test";

/**
 * Comparing an item's prices at Woolworths and Coles, in a real browser
 * against the real backend.
 *
 * The backend must run with `STORE_CLIENTS=fake`: these tests read the fake
 * catalogue (`backend/src/services/stores/fake.rs`), never a real store.
 */

type View = ReturnType<Page["getByRole"]>;

/** The product names shown, top to bottom; only one store's when given. */
async function productNames(view: View, store?: "woolworths" | "coles") {
	const list = view.getByRole("list", { name: "Products" });
	await expect(list).toBeVisible();
	const rows = store
		? list.locator(`[data-testid="product-listing"][data-store="${store}"]`)
		: list.getByRole("listitem");
	return rows
		.getByRole("article")
		.evaluateAll((articles) => articles.map((a) => a.getAttribute("aria-label")));
}

/** Picks one way to sort the comparison. */
async function sortBy(page: Page, view: View, label: string) {
	await view.getByRole("combobox", { name: "Sort by" }).click();
	await page.getByRole("option", { name: label, exact: true }).click();
}

test.describe("price comparison", () => {
	test("shows both stores together, cheapest per unit first", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");

		const view = await openSelectItem(page, "milk");

		await expect(
			view.getByRole("heading", { name: "Select milk ×1" }),
		).toBeVisible();
		// Per 100mL: 15.5c, 16.5c (Coles), 18c, 39c (Coles).
		expect(await productNames(view)).toEqual([
			"Woolworths Full Cream Milk",
			"Coles Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
			"Minor Figures Barista Oat Milk",
		]);
		const cheapest = view.getByRole("article", {
			name: "Woolworths Full Cream Milk",
		});
		await expect(cheapest).toContainText("$3.10");
		await expect(cheapest).toContainText("$0.155 / 100mL");
		await expect(cheapest).toContainText("unit price calculated from 1L");
		const picture = cheapest.getByRole("img", {
			name: "Woolworths Full Cream Milk",
		});
		await expect(picture).toBeVisible();
		// Loaded, not broken: a broken image has no natural width.
		expect(
			await picture.evaluate((img: HTMLImageElement) => img.naturalWidth),
		).toBeGreaterThan(0);
	});

	test("sorts by unit price, price and name, both ways", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const view = await openSelectItem(page, "milk");
		const byUnit = [
			"Woolworths Full Cream Milk",
			"Coles Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
			"Minor Figures Barista Oat Milk",
		];
		const byPrice = [
			"Woolworths Full Cream Milk",
			"Minor Figures Barista Oat Milk",
			"Coles Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
		];
		const byName = [
			"Coles Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
			"Minor Figures Barista Oat Milk",
			"Woolworths Full Cream Milk",
		];

		await sortBy(page, view, "Unit price: high to low");
		expect(await productNames(view)).toEqual([...byUnit].reverse());
		await sortBy(page, view, "Price: low to high");
		expect(await productNames(view)).toEqual(byPrice);
		await sortBy(page, view, "Price: high to low");
		expect(await productNames(view)).toEqual([...byPrice].reverse());
		await sortBy(page, view, "Name: A to Z");
		expect(await productNames(view)).toEqual(byName);
		await sortBy(page, view, "Name: Z to A");
		expect(await productNames(view)).toEqual([...byName].reverse());
		await sortBy(page, view, "Unit price: low to high");
		expect(await productNames(view)).toEqual(byUnit);
	});

	test("the item's filters narrow the products", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "toilet paper");
		const card = itemCard(page, "toilet paper");
		await card.getByRole("button", { name: "Edit toilet paper" }).click();
		await card.getByLabel("Add a product filter").fill("3 ply");
		await card.getByLabel("Add a product filter").press("Enter");
		await card.getByRole("button", { name: "Save" }).click();
		await expect(card.getByLabel("Product filters")).toContainText("3 ply");

		const view = await openSelectItem(page, "toilet paper");

		expect(await productNames(view, "woolworths")).toEqual([
			"Kleenex Complete Clean 3 Ply Toilet Paper",
		]);
		expect(await productNames(view, "coles")).toEqual([
			"Quilton 3 Ply Toilet Rolls",
		]);
	});

	test("specials only hides products with no promotion", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const view = await openSelectItem(page, "milk");
		await expect(view.getByRole("list", { name: "Products" })).toBeVisible();

		await view.getByRole("switch", { name: "Specials only" }).click();

		expect(await productNames(view)).toEqual([
			"Dairy Farmers Full Cream Milk",
			"Minor Figures Barista Oat Milk",
		]);
	});

	test("a store that cannot be reached is explained, the other still shows", async ({
		page,
	}) => {
		await gotoList(page);
		// The fake Coles fails any search containing "outage".
		await addItem(page, "milk outage");

		const view = await openSelectItem(page, "milk outage");

		await expect(view.getByRole("status")).toHaveText(
			"Coles could not be reached. Try again later.",
		);
		expect((await productNames(view, "woolworths")).length).toBeGreaterThan(0);
		expect(await productNames(view, "coles")).toEqual([]);
	});

	test("works on a committed list, and goes back to it", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "bananas");
		await page.getByRole("button", { name: "Ready to order" }).click();
		await expect(itemCard(page, "bananas")).toContainText("Committed");

		const view = await openSelectItem(page, "bananas");
		await expect(
			view.getByRole("article", { name: "Woolworths Cavendish Bananas" }),
		).toContainText("unit price calculated from 1kg");

		await backToList(page);
		await expect(itemCard(page, "bananas")).toBeVisible();
	});

	test("filters by store and size", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const view = await openSelectItem(page, "milk");

		await view.getByRole("combobox", { name: "Store" }).click();
		await page.getByRole("option", { name: "Coles" }).click();
		expect(await productNames(view, "woolworths")).toEqual([]);
		expect((await productNames(view, "coles")).length).toBeGreaterThan(0);

		await view.getByRole("combobox", { name: "Size" }).click();
		await page.getByRole("option", { name: "Grams" }).click();
		await expect(view.getByRole("list", { name: "Products" })).toHaveCount(0);
	});

	test("Settings › Display changes the picture size and hides old prices", async ({
		page,
	}) => {
		await gotoList(page, "/settings/display");
		await page.getByRole("combobox", { name: "Picture size" }).click();
		await page.getByRole("option", { name: "Large" }).click();
		await page.getByRole("switch", { name: "Do not display original price" }).click();
		await expect(page.getByText("Display saved").first()).toBeVisible();

		await gotoList(page);
		await addItem(page, "milk");
		const view = await openSelectItem(page, "milk");
		const picture = view
			.getByRole("article", { name: "Woolworths Full Cream Milk" })
			.getByRole("img", { name: "Woolworths Full Cream Milk" });
		expect(await picture.evaluate((img) => (img as HTMLElement).offsetWidth)).toBe(96);
		await expect(view.locator("s")).toHaveCount(0);
	});
});
