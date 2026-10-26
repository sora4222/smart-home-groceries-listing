import type { Page } from "@playwright/test";
import {
	addItem,
	backToList,
	expect,
	gotoList,
	itemCard,
	openSelectItem,
	test,
} from "./fixtures";

/** Clicks Undo on the toast that says `text`. */
async function undo(page: Page, text: string) {
	await page
		.locator("[data-sonner-toast]")
		.filter({ hasText: text })
		.getByRole("button", { name: "Undo" })
		.click();
	await expect(page.getByText("Undone")).toBeVisible();
}

/** Every change on the grocery list can be undone from its toast. */
test.describe("undo on the grocery list", () => {
	test("adding an item", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");

		await undo(page, "milk is on the list");

		await expect(itemCard(page, "milk")).toHaveCount(0);
	});

	test("removing an item brings it back with its product", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 2);
		const view = await openSelectItem(page, "milk");
		await view
			.getByRole("button", { name: "Choose Coles Full Cream Milk at Coles" })
			.click();
		await backToList(page);

		await page.getByLabel("Remove milk").click();
		await page.getByLabel("Confirm removing milk").click();
		await expect(itemCard(page, "milk")).toHaveCount(0);
		await undo(page, "Item removed");

		await expect(itemCard(page, "milk").getByLabel("Quantity 2")).toBeVisible();
		await expect(
			itemCard(page, "milk").getByLabel("Chosen product for milk", {
				exact: true,
			}),
		).toContainText("Coles Full Cream Milk");
	});

	test("editing an item", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const card = itemCard(page, "milk");
		await card.getByLabel("Edit milk").click();
		await card.getByLabel("Item", { exact: true }).fill("oat milk");
		await card.getByRole("button", { name: "Save" }).click();
		await expect(itemCard(page, "oat milk")).toBeVisible();

		await undo(page, "Saved");

		await expect(itemCard(page, "milk")).toBeVisible();
	});

	test("committing the list", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		await page.getByRole("button", { name: "Ready to order (1 item)" }).click();
		await expect(page.getByLabel("Edit milk")).toHaveCount(0);

		await undo(page, "1 item ready to order");

		await expect(page.getByLabel("Edit milk")).toBeVisible();
	});
});
