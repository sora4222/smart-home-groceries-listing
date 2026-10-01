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

/** The product names shown for one store, top to bottom. */
async function productsAt(sheet: ReturnType<Page["getByRole"]>, store: string) {
	const region = sheet.getByRole("region", { name: store });
	await expect(region).toBeVisible();
	return region.getByRole("article").evaluateAll((articles) =>
		articles.map((a) => a.getAttribute("aria-label")),
	);
}

test.describe("price comparison", () => {
	test("shows both stores, cheapest per unit first", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");

		const sheet = await openComparison(page, "milk");

		await expect(
			sheet.getByRole("heading", { name: "Prices for milk ×1" }),
		).toBeVisible();
		// $1.55/L for the 2L beats $1.80/L for the 3L.
		expect(await productsAt(sheet, "Woolworths")).toEqual([
			"Woolworths Full Cream Milk",
			"Dairy Farmers Full Cream Milk",
		]);
		const cheapest = sheet.getByRole("article", {
			name: "Woolworths Full Cream Milk",
		});
		await expect(cheapest).toContainText("$3.10");
		await expect(cheapest).toContainText("$0.155 / 100mL");
		await expect(cheapest).toContainText("unit price calculated from 1L");
		await expect(sheet.getByRole("region", { name: "Coles" })).toBeVisible();
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

		expect(await productsAt(sheet, "Woolworths")).toEqual([
			"Kleenex Complete Clean 3 Ply Toilet Paper",
		]);
		expect(await productsAt(sheet, "Coles")).toEqual([
			"Quilton 3 Ply Toilet Rolls",
		]);
	});

	test("specials only hides products with no promotion", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await expect(sheet.getByRole("region", { name: "Coles" })).toBeVisible();

		await sheet.getByRole("switch", { name: "Specials only" }).click();

		expect(await productsAt(sheet, "Woolworths")).toEqual([
			"Dairy Farmers Full Cream Milk",
		]);
		expect(await productsAt(sheet, "Coles")).toEqual([
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

		await expect(
			sheet.getByRole("region", { name: "Coles" }).getByRole("status"),
		).toHaveText("Coles could not be reached. Try again later.");
		expect((await productsAt(sheet, "Woolworths")).length).toBeGreaterThan(0);
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
