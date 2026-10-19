import {
	addItem,
	deliverVoiceItem,
	expect,
	gotoList,
	itemCard,
	test,
} from "./fixtures";

/**
 * Accepting a voice item whose name is already on the list. The backend
 * answers 409 with the existing item; the Pending Requests screen must ask
 * "update the quantity or add a separate entry?" rather than show an error.
 */
test.describe("accepting a voice item already on the list", () => {
	test("updates the existing quantity when asked to", async ({ page, request }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);
		expect(await deliverVoiceItem(request, "milk", 2)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept milk" }).click();

		const prompt = page.getByRole("alertdialog", { name: "Already on the list" });
		await expect(prompt).toContainText("milk is already on the list");
		await prompt.getByRole("button", { name: "Update the quantity" }).click();
		await expect(page.getByRole("button", { name: "Accept milk" })).toHaveCount(0);

		await gotoList(page);
		await expect(itemCard(page, "milk")).toHaveCount(1);
		await expect(itemCard(page, "milk").getByLabel("Quantity 3")).toBeVisible();
	});

	test("adds a separate entry when asked to", async ({ page, request }) => {
		await gotoList(page);
		await addItem(page, "bread", 1);
		expect(await deliverVoiceItem(request, "bread", 1)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept bread" }).click();
		await page.getByRole("button", { name: "Add a separate entry" }).click();
		await expect(page.getByRole("button", { name: "Accept bread" })).toHaveCount(0);

		await gotoList(page);
		await expect(itemCard(page, "bread")).toHaveCount(2);
	});

	test("cancel keeps the request waiting", async ({ page, request }) => {
		await gotoList(page);
		await addItem(page, "eggs", 1);
		expect(await deliverVoiceItem(request, "eggs", 1)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept eggs" }).click();
		await page.getByRole("button", { name: "Cancel" }).click();

		await expect(page.getByRole("alertdialog")).toHaveCount(0);
		await expect(page.getByRole("button", { name: "Accept eggs" })).toBeVisible();
	});

	test("editing the card closes the question about the old name", async ({
		page,
		request,
	}) => {
		await gotoList(page);
		await addItem(page, "jam", 1);
		expect(await deliverVoiceItem(request, "jam", 1)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept jam" }).click();
		await expect(page.getByRole("alertdialog")).toBeVisible();
		await page.getByLabel("Item name").fill("strawberry jam");

		await expect(page.getByRole("alertdialog")).toHaveCount(0);
		await page.getByRole("button", { name: "Accept strawberry jam" }).click();
		await expect(
			page.getByRole("button", { name: "Accept strawberry jam" }),
		).toHaveCount(0);
	});
});
