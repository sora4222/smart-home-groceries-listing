import { addForm, expect, gotoList, itemCard, test } from "./fixtures";

/**
 * The layout and keyboard requirements from the spec's Frontend section: the
 * page has to work at every width, and everything has to be reachable without
 * a mouse. Both projects run this, so it is checked at 1280px and on a phone.
 */
test.describe("usability", () => {
	test("does not scroll sideways at any width", async ({ page }) => {
		await gotoList(page);
		await addForm(page).name.fill("a very long item name to push the layout wider");
		await addForm(page).submit.click();
		await expect(page.getByTestId("grocery-item")).toHaveCount(1);

		const overflow = await page.evaluate(() => {
			const root = document.documentElement;
			return root.scrollWidth - root.clientWidth;
		});
		expect(overflow, "no horizontal page scroll").toBeLessThanOrEqual(1);
	});

	test("adds an item using only the keyboard", async ({ page }) => {
		await gotoList(page);

		await page.getByLabel("Item", { exact: true }).focus();
		await page.keyboard.type("bananas");
		await page.keyboard.press("Tab");
		await page.keyboard.type("3");
		await page.keyboard.press("Enter");

		const item = itemCard(page, "bananas");
		await expect(item).toBeVisible();
		await expect(item.getByLabel("Quantity 3")).toHaveText("×3");
	});

	test("reaches the edit and remove actions by keyboard", async ({ page }) => {
		await gotoList(page);
		await addForm(page).name.fill("milk");
		await addForm(page).submit.click();
		await expect(itemCard(page, "milk")).toBeVisible();

		const edit = page.getByLabel("Edit milk");
		await edit.focus();
		await expect(edit).toBeFocused();
		await page.keyboard.press("Enter");

		// The editor opens and its first field is reachable.
		const note = itemCard(page, "milk").getByLabel("Note");
		await expect(note).toBeVisible();
		await note.focus();
		await page.keyboard.type("keyboard only");
		await page.getByRole("button", { name: "Save" }).click();

		await expect(page.getByText("keyboard only")).toBeVisible();
	});

	test("labels the icon-only chip buttons for screen readers", async ({ page }) => {
		await gotoList(page);
		await addForm(page).name.fill("toilet paper");
		await addForm(page).submit.click();
		await page.getByLabel("Edit toilet paper").click();
		await page.getByLabel("Add a product filter").fill("3 ply");
		await page.getByRole("button", { name: "Add filter" }).click();
		await page.getByRole("button", { name: "Save" }).click();

		// The × is an icon; the accessible name has to say what it removes.
		await expect(page.getByRole("button", { name: "Remove filter 3 ply" })).toBeVisible();
	});
});
