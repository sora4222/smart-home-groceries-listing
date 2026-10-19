import { addItem, expect, gotoList, itemCard, test } from "./fixtures";
import type { Page } from "@playwright/test";

/**
 * Comparing an item's prices at Woolworths and Coles, in a real browser
 * against the real backend.
 *
 * The backend must run with `STORE_CLIENTS=fake`: these tests read the fake
 * catalogue (`backend/src/services/stores/fake.rs`), never a real store.
 */

/** Opens the comparison for one item and returns its sheet. */
async function openComparison(page: Page, name: string) {
	await itemCard(page, name)
		.getByRole("button", { name: `Compare prices for ${name}` })
		.click();
	const sheet = page.getByRole("dialog");
	await expect(sheet).toBeVisible();
	return sheet;
}

type Sheet = ReturnType<Page["getByRole"]>;

/** The product names shown, top to bottom; only one store's when given. */
async function productNames(sheet: Sheet, store?: "woolworths" | "coles") {
	const list = sheet.getByRole("list", { name: "Products" });
	await expect(list).toBeVisible();
	const rows = store
		? list.locator(`[data-testid="product-listing"][data-store="${store}"]`)
		: list.getByRole("listitem");
	return rows
		.getByRole("article")
		.evaluateAll((articles) => articles.map((a) => a.getAttribute("aria-label")));
}

/** Picks one way to sort the comparison. */
async function sortBy(page: Page, sheet: Sheet, label: string) {
	await sheet.getByRole("combobox", { name: "Sort by" }).click();
	await page.getByRole("option", { name: label, exact: true }).click();
}

test.describe("price comparison", () => {
	test("shows both stores together, cheapest per unit first", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");

		const sheet = await openComparison(page, "milk");

		await expect(
			sheet.getByRole("heading", { name: "Prices for milk ×1" }),
		).toBeVisible();
		// Per 100mL: 15.5c, 16.5c (Coles), 18c, 39c (Coles).
		expect(await productNames(sheet)).toEqual([
			"Woolworths Full Cream Milk",
			"Coles Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
			"Minor Figures Barista Oat Milk",
		]);
		const cheapest = sheet.getByRole("article", {
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
		const sheet = await openComparison(page, "milk");
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

		await sortBy(page, sheet, "Unit price: high to low");
		expect(await productNames(sheet)).toEqual([...byUnit].reverse());
		await sortBy(page, sheet, "Price: low to high");
		expect(await productNames(sheet)).toEqual(byPrice);
		await sortBy(page, sheet, "Price: high to low");
		expect(await productNames(sheet)).toEqual([...byPrice].reverse());
		await sortBy(page, sheet, "Name: A to Z");
		expect(await productNames(sheet)).toEqual(byName);
		await sortBy(page, sheet, "Name: Z to A");
		expect(await productNames(sheet)).toEqual([...byName].reverse());
		await sortBy(page, sheet, "Unit price: low to high");
		expect(await productNames(sheet)).toEqual(byUnit);
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

		const sheet = await openComparison(page, "toilet paper");

		expect(await productNames(sheet, "woolworths")).toEqual([
			"Kleenex Complete Clean 3 Ply Toilet Paper",
		]);
		expect(await productNames(sheet, "coles")).toEqual([
			"Quilton 3 Ply Toilet Rolls",
		]);
	});

	test("specials only hides products with no promotion", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await expect(sheet.getByRole("list", { name: "Products" })).toBeVisible();

		await sheet.getByRole("switch", { name: "Specials only" }).click();

		expect(await productNames(sheet)).toEqual([
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

		const sheet = await openComparison(page, "milk outage");

		await expect(sheet.getByRole("status")).toHaveText(
			"Coles could not be reached. Try again later.",
		);
		expect((await productNames(sheet, "woolworths")).length).toBeGreaterThan(0);
		expect(await productNames(sheet, "coles")).toEqual([]);
	});

	test("closes with Escape and works on a committed list", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "bananas");
		await page.getByRole("button", { name: "Ready to order" }).click();
		await expect(itemCard(page, "bananas")).toContainText("Committed");

		const sheet = await openComparison(page, "bananas");
		await expect(
			sheet.getByRole("article", { name: "Woolworths Cavendish Bananas" }),
		).toContainText("unit price calculated from 1kg");

		await page.keyboard.press("Escape");
		await expect(sheet).toBeHidden();
	});
});
