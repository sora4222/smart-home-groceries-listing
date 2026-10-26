import {
	addItem,
	deliverVoiceItem,
	expect,
	gotoList,
	itemCard,
	test,
} from "./fixtures";

/**
 * Accepting from Pending. An item already on the list gets the quantity added
 * to it, with no question; Undo takes it back and returns the card.
 */
test.describe("accepting a pending request", () => {
	test("adds to the quantity of an item already on the list", async ({
		page,
		request,
	}) => {
		await gotoList(page);
		await addItem(page, "milk", 1);
		expect(await deliverVoiceItem(request, "milk", 2)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept milk" }).click();
		await expect(page.getByText("milk: now 3")).toBeVisible();
		await expect(page.getByText("No items pending")).toBeVisible();

		await gotoList(page);
		await expect(itemCard(page, "milk")).toHaveCount(1);
		await expect(itemCard(page, "milk").getByLabel("Quantity 3")).toBeVisible();
	});

	test("Undo takes the added quantity back and returns the card", async ({
		page,
		request,
	}) => {
		await gotoList(page);
		await addItem(page, "eggs", 1);
		expect(await deliverVoiceItem(request, "eggs", 2)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept eggs" }).click();
		await page.getByRole("button", { name: "Undo" }).click();
		await expect(page.getByRole("button", { name: "Accept eggs" })).toBeVisible();

		await gotoList(page);
		await expect(itemCard(page, "eggs").getByLabel("Quantity 1")).toBeVisible();
	});

	test("a new item is added, and Undo removes it", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "jam", 1)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept jam" }).click();
		await expect(page.getByText("jam added")).toBeVisible();
		await page.getByRole("button", { name: "Undo" }).click();
		await expect(page.getByRole("button", { name: "Accept jam" })).toBeVisible();

		await gotoList(page);
		await expect(itemCard(page, "jam")).toHaveCount(0);
	});

	test("the nav's Pending link opens the page", async ({ page }) => {
		await gotoList(page);
		await page
			.getByRole("navigation", { name: "Main navigation" })
			.getByRole("link", { name: "Pending" })
			.click();
		await expect(page.getByRole("heading", { name: "Pending" })).toBeVisible();
		await expect(page.getByText("No items pending")).toBeVisible();
	});
});
