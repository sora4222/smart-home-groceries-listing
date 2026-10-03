import type { Page } from "@playwright/test";
import { addItem, expect, gotoList, itemCard, test } from "./fixtures";

/**
 * Choosing one product for each list item, in a real browser against the
 * real backend.
 *
 * The backend must run with `STORE_CLIENTS=fake`: these tests read the fake
 * catalogue (`backend/src/services/stores/fake.rs`), never a real store.
 */

/** Opens the price comparison for one item and returns its sheet. */
async function openComparison(page: Page, name: string) {
	await itemCard(page, name)
		.getByRole("button", { name: `Compare prices for ${name}` })
		.click();
	const sheet = page.getByRole("dialog");
	await expect(sheet).toBeVisible();
	await expect(sheet.getByRole("region", { name: "Coles" })).toBeVisible();
	return sheet;
}

/** The chosen-product panel on an item's card. */
function chosenOn(page: Page, name: string) {
	return itemCard(page, name).getByLabel(`Chosen product for ${name}`, { exact: true });
}

test.describe("choosing a product", () => {
	test("a new item has no product chosen yet", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");

		await expect(itemCard(page, "milk")).toContainText("No product chosen yet");
	});

	test("choosing a product shows it on the item and in the sheet", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");

		await sheet
			.getByRole("button", { name: "Choose Coles Full Cream Milk at Coles" })
			.click();

		const chosen = sheet.getByRole("button", {
			name: "Coles Full Cream Milk at Coles is chosen",
		});
		await expect(chosen).toHaveAttribute("aria-pressed", "true");
		await page.keyboard.press("Escape");
		await expect(chosenOn(page, "milk")).toContainText("Coles Full Cream Milk");
		await expect(chosenOn(page, "milk")).toContainText("3L · Coles · $4.95 each");
	});

	test("choosing at the other store keeps both, and the newest is bought", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await sheet
			.getByRole("button", { name: "Choose Coles Full Cream Milk at Coles" })
			.click();
		await expect(
			sheet.getByRole("button", { name: /Coles Full Cream Milk at Coles is chosen/ }),
		).toBeVisible();

		await sheet
			.getByRole("button", {
				name: "Choose Woolworths Full Cream Milk at Woolworths",
			})
			.click();

		// One product per store: both stay chosen.
		await expect(sheet.getByRole("button", { name: /is chosen$/ })).toHaveCount(2);
		await page.keyboard.press("Escape");
		await expect(chosenOn(page, "milk")).toContainText("Woolworths");
		await expect(chosenOn(page, "milk")).toContainText("$3.10 each");
		await expect(
			page.getByRole("region", { name: "Also chosen at Coles for milk" }),
		).toContainText("$4.95 each");
	});

	test("the choice survives a reload and can be cleared", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "bananas");
		const sheet = await openComparison(page, "bananas");
		await sheet
			.getByRole("button", { name: "Choose Coles Bananas at Coles" })
			.click();
		await page.keyboard.press("Escape");
		await expect(chosenOn(page, "bananas")).toBeVisible();

		await gotoList(page);
		await expect(chosenOn(page, "bananas")).toContainText("Coles Bananas");

		await itemCard(page, "bananas")
			.getByRole("button", { name: "Clear the chosen product for bananas" })
			.click();
		await expect(itemCard(page, "bananas")).toContainText(
			"No product chosen yet",
		);
	});

	test("renaming the item drops its choice", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await sheet
			.getByRole("button", { name: "Choose Coles Full Cream Milk at Coles" })
			.click();
		await page.keyboard.press("Escape");
		await expect(chosenOn(page, "milk")).toBeVisible();

		const card = itemCard(page, "milk");
		await card.getByRole("button", { name: "Edit milk" }).click();
		await card.getByLabel("Item", { exact: true }).fill("oat milk");
		await card.getByRole("button", { name: "Save" }).click();

		await expect(itemCard(page, "oat milk")).toContainText(
			"No product chosen yet",
		);
	});

	test("a product can be chosen on a committed list", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "toilet paper");
		await page.getByRole("button", { name: "Ready to order" }).click();
		await expect(itemCard(page, "toilet paper")).toContainText("Committed");

		const sheet = await openComparison(page, "toilet paper");
		await sheet
			.getByRole("button", {
				name: "Choose Quilton 3 Ply Toilet Rolls at Coles",
			})
			.click();
		await page.keyboard.press("Escape");

		await expect(chosenOn(page, "toilet paper")).toContainText(
			"Quilton 3 Ply Toilet Rolls",
		);
	});
});
