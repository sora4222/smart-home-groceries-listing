import { addItem, backToList, expect, gotoList, itemCard, openSelectItem, test } from "./fixtures";

/**
 * Buying several products for one item, in a real browser against the real
 * backend with `STORE_CLIENTS=fake`. Each product has a count; the counts
 * add up to the item's quantity.
 */

const SMALL = "Woolworths Full Cream Milk 2L";
const LARGE = "Dairy Farmers Full Cream Milk 3L";

test.describe("several products for one item", () => {
	test("two products share the quantity, and a count can change with Undo", async ({
		page,
	}) => {
		await gotoList(page);
		await addItem(page, "milk", 3);
		const view = await openSelectItem(page, "milk");
		await view
			.getByRole("button", { name: "Choose Woolworths Full Cream Milk at Woolworths" })
			.click();
		await expect(view.getByRole("region", { name: "Chosen product for milk" })).toContainText("×3");
		await view
			.getByRole("button", { name: "Choose Dairy Farmers Full Cream Milk at Woolworths" })
			.click();

		// The new product takes one from the first.
		await expect(view.getByRole("combobox", { name: `How many ${SMALL}` })).toHaveText("×2");
		await expect(view.getByRole("combobox", { name: `How many ${LARGE}` })).toHaveText("×1");

		await backToList(page);
		const card = itemCard(page, "milk");
		await card.getByRole("combobox", { name: `How many ${LARGE}` }).click();
		await page.getByRole("option", { name: "×2" }).click();
		await expect(card.getByRole("combobox", { name: `How many ${SMALL}` })).toHaveText("×1");
		await expect(card.getByRole("combobox", { name: `How many ${LARGE}` })).toHaveText("×2");

		await page
			.locator("[data-sonner-toast]")
			.filter({ hasText: "Count changed" })
			.getByRole("button", { name: "Undo" })
			.click();
		await expect(card.getByRole("combobox", { name: `How many ${SMALL}` })).toHaveText("×2");
		await expect(card.getByRole("combobox", { name: `How many ${LARGE}` })).toHaveText("×1");
	});

	test("clearing one product gives its count back to the other", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 2);
		const view = await openSelectItem(page, "milk");
		await view
			.getByRole("button", { name: "Choose Woolworths Full Cream Milk at Woolworths" })
			.click();
		await view
			.getByRole("button", { name: "Choose Dairy Farmers Full Cream Milk at Woolworths" })
			.click();
		await expect(view.getByRole("region", { name: `Also buying ${LARGE} for milk` })).toBeVisible();

		await backToList(page);
		const card = itemCard(page, "milk");
		await card.getByRole("button", { name: `Clear ${LARGE} for milk` }).click();

		await expect(card.getByRole("region", { name: "Chosen product for milk" })).toContainText("×2");
		await expect(card.getByRole("combobox")).toHaveCount(0);
	});

	test("tapping Chosen again un-chooses the product, with Undo", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		const view = await openSelectItem(page, "milk");
		await view
			.getByRole("button", { name: "Choose Woolworths Full Cream Milk at Woolworths" })
			.click();

		await view
			.getByRole("button", { name: "Woolworths Full Cream Milk at Woolworths is chosen" })
			.click();
		await expect(
			view.getByRole("button", { name: "Choose Woolworths Full Cream Milk at Woolworths" }),
		).toBeVisible();

		await page
			.locator("[data-sonner-toast]")
			.filter({ hasText: "Chosen product cleared" })
			.getByRole("button", { name: "Undo" })
			.click();
		await expect(
			view.getByRole("button", { name: "Woolworths Full Cream Milk at Woolworths is chosen" }),
		).toBeVisible();
	});
});
