import type { Page } from "@playwright/test";
import { addItem, expect, gotoList, itemCard, test } from "./fixtures";

/**
 * Disliking products, in a real browser against the real backend.
 *
 * The backend must run with `STORE_CLIENTS=fake` and `DEV_AUTH_BYPASS=true`:
 * every request is the same dev user, so every dislike here is "mine".
 */

const COLES_MILK = "Coles Full Cream Milk at Coles";

/** Opens the price comparison for one item and returns its sheet. */
async function openComparison(page: Page, name: string) {
	await itemCard(page, name)
		.getByRole("button", { name: `Compare prices for ${name}` })
		.click();
	const sheet = page.getByRole("dialog");
	await expect(sheet.getByRole("region", { name: "Coles" })).toBeVisible();
	return sheet;
}

/** The Coles milk's row in the sheet. */
function colesMilk(page: Page) {
	return page
		.getByRole("dialog")
		.locator('[data-testid="store-product"][data-product-id="c-milk-3l"]');
}

test.describe("disliking a product", () => {
	test("a disliked product shows a warning and can still be chosen", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");

		await sheet.getByRole("button", { name: `Dislike ${COLES_MILK}` }).click();

		await expect(colesMilk(page).getByRole("status")).toHaveText(
			"You disliked this item previously.",
		);
		await sheet.getByRole("button", { name: `Choose ${COLES_MILK}` }).click();
		await expect(
			sheet.getByRole("button", { name: `${COLES_MILK} is chosen` }),
		).toBeVisible();
	});

	test("removing my dislike takes the warning away", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await sheet.getByRole("button", { name: `Dislike ${COLES_MILK}` }).click();
		await expect(colesMilk(page).getByTestId("dislike-notice")).toBeVisible();

		await sheet
			.getByRole("button", { name: `Remove my dislike of ${COLES_MILK}` })
			.click();

		await expect(colesMilk(page).getByTestId("dislike-notice")).toHaveCount(0);
	});

	test("a dislike can be set aside for this order and brought back", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await sheet.getByRole("button", { name: `Dislike ${COLES_MILK}` }).click();

		await colesMilk(page).getByRole("button", { name: "Buy it this time" }).click();
		await expect(colesMilk(page)).toContainText("OK to buy it this time.");
		await expect(colesMilk(page).getByRole("status")).toBeVisible();

		await colesMilk(page).getByRole("button", { name: "Undo" }).click();
		await expect(
			colesMilk(page).getByRole("button", { name: "Buy it this time" }),
		).toBeVisible();
	});

	test("the Dislikes page lists my dislikes and can remove them", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk");
		const sheet = await openComparison(page, "milk");
		await sheet.getByRole("button", { name: `Dislike ${COLES_MILK}` }).click();
		await expect(colesMilk(page).getByTestId("dislike-notice")).toBeVisible();

		await gotoList(page, "/settings/dislikes");
		const mine = page.getByRole("region", { name: "Your dislikes" });
		await expect(mine).toContainText("Coles Full Cream Milk");
		await expect(mine).toContainText("3L · Coles");

		await mine
			.getByRole("button", { name: `Remove my dislike of ${COLES_MILK}` })
			.click();
		await expect(page.getByText(/Nobody dislikes a product yet/)).toBeVisible();
	});
});
