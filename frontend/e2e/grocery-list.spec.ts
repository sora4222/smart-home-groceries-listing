import {
	deliverVoiceItem,
	addForm,
	addItem,
	expect,
	gotoList,
	itemCard,
	test,
} from "./fixtures";

/**
 * The grocery list in a real browser, against the real Rust backend and a real
 * database: read the list, add to it, review and annotate items, and commit
 * the list for purchase.
 */
test.describe("grocery list", () => {
	test("shows an empty list with a way in", async ({ page }) => {
		await gotoList(page);

		await expect(page.getByRole("heading", { name: "Grocery List" })).toBeVisible();
		await expect(page.getByText("Nothing on the list yet.")).toBeVisible();
		await expect(page.getByRole("button", { name: "Add item" })).toBeVisible();
	});

	test("adds an item and keeps it after a reload", async ({ page }) => {
		await gotoList(page);

		await addItem(page, "milk", 2);

		const item = itemCard(page, "milk");
		await expect(item.getByLabel("Quantity 2")).toHaveText("×2");
		await expect(item).toContainText("Added via the web app");

		// It is on the server, not only in this tab.
		await page.reload();
		await expect(itemCard(page, "milk")).toBeVisible();
	});

	test("adds several items and lists them all", async ({ page }) => {
		await gotoList(page);

		await addItem(page, "bread");
		await addItem(page, "bananas");
		await addItem(page, "coffee");

		await expect(page.getByTestId("grocery-item")).toHaveCount(3);
		// The form empties itself for the next item.
		await expect(addForm(page).name).toHaveValue("");
	});

	test("asks what to do about a name already on the list", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);

		const form = addForm(page);
		await form.name.fill("  MILK ");
		await form.quantity.fill("2");
		await form.submit.click();

		await expect(page.getByText(/already on the list/)).toBeVisible();
		await expect(page.getByTestId("grocery-item")).toHaveCount(1);
	});

	test("updates the existing quantity when asked to", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);

		const form = addForm(page);
		await form.name.fill("milk");
		await form.quantity.fill("2");
		await form.submit.click();
		await page.getByRole("button", { name: "Update the quantity" }).click();

		await expect(page.getByTestId("grocery-item")).toHaveCount(1);
		await expect(page.getByLabel("Quantity 3")).toHaveText("×3");
	});

	test("keeps both entries when asked for a separate one", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);

		const form = addForm(page);
		await form.name.fill("milk");
		await form.submit.click();
		await page.getByRole("button", { name: "Add a separate entry" }).click();

		await expect(page.getByTestId("grocery-item")).toHaveCount(2);
	});

	test("edits an item's name, quantity and note", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);

		const editor = itemCard(page, "milk");
		await page.getByLabel("Edit milk").click();
		await editor.getByLabel("Item", { exact: true }).fill("full cream milk");
		await editor.getByLabel("Quantity", { exact: true }).fill("3");
		await editor.getByLabel("Note").fill("the 2 litre bottle");
		await page.getByRole("button", { name: "Save" }).click();

		const item = itemCard(page, "full cream milk");
		await expect(item).toBeVisible();
		await expect(item.getByLabel("Quantity 3")).toHaveText("×3");
		await expect(item).toContainText("the 2 litre bottle");

		await page.reload();
		await expect(page.getByText("the 2 litre bottle")).toBeVisible();
	});

	test("annotates an item with filter chips and takes one off again", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "toilet paper");

		await page.getByLabel("Edit toilet paper").click();
		await page.getByLabel("Add a product filter").fill("3 ply");
		await page.getByRole("button", { name: "Add filter" }).click();
		await page.getByLabel("Add a product filter").fill("recycled");
		await page.getByRole("button", { name: "Add filter" }).click();
		await page.getByRole("button", { name: "Save" }).click();

		const filters = page.getByLabel("Product filters");
		await expect(filters).toContainText("3 ply");
		await expect(filters).toContainText("recycled");

		// Chips come off from the list itself, without opening the editor.
		await page.getByLabel("Remove filter 3 ply").click();
		await expect(page.getByLabel("Product filters")).not.toContainText("3 ply");
		await expect(page.getByLabel("Product filters")).toContainText("recycled");

		await page.reload();
		await expect(page.getByLabel("Product filters")).toContainText("recycled");
	});

	test("leaves an item alone when an edit is cancelled", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk", 1);

		const card = itemCard(page, "milk");
		await page.getByLabel("Edit milk").click();
		await card.getByLabel("Item", { exact: true }).fill("oat milk");
		await page.getByRole("button", { name: "Cancel" }).click();

		await expect(card).toContainText("milk");
		await expect(page.getByText("oat milk")).toHaveCount(0);
	});

	test("asks before removing an item, then removes it", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");

		await page.getByLabel("Remove milk").click();
		await page.getByRole("button", { name: "Keep" }).click();
		await expect(page.getByTestId("grocery-item")).toHaveCount(1);

		await page.getByLabel("Remove milk").click();
		await page.getByLabel("Confirm removing milk").click();

		await expect(page.getByText("Nothing on the list yet.")).toBeVisible();
		await expect(page.getByTestId("grocery-item")).toHaveCount(0);
	});
});

test.describe("committing the list", () => {
	test("locks the list in and releases it again", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		await addItem(page, "bread");

		await page.getByRole("button", { name: "Ready to order (2 items)" }).click();

		await expect(page.getByText("2 items locked in for purchase.")).toBeVisible();
		await expect(page.getByRole("heading", { name: "Committed for purchase" })).toBeVisible();
		await expect(page.getByText("Committed").first()).toBeVisible();

		// A committed item is read-only.
		await expect(page.getByLabel("Edit milk")).toHaveCount(0);
		await expect(page.getByLabel("Remove milk")).toHaveCount(0);

		await page.getByRole("button", { name: "Release for editing" }).click();

		await expect(page.getByLabel("Edit milk")).toBeVisible();
		await expect(page.getByText("locked in for purchase.")).toHaveCount(0);
	});

	test("an item added after a commit starts the next list", async ({ page }) => {
		await gotoList(page);
		await addItem(page, "milk");
		await page.getByRole("button", { name: "Ready to order (1 item)" }).click();
		await expect(page.getByText("1 item locked in for purchase.")).toBeVisible();

		await addItem(page, "bread");

		await expect(page.getByText("1 item locked in for purchase.")).toBeVisible();
		await expect(page.getByRole("button", { name: "Ready to order (1 item)" })).toBeVisible();
		await expect(page.getByLabel("Edit bread")).toBeVisible();
	});
});

test.describe("a voice item reaching the list", () => {
	test("appears on the main page once it is accepted", async ({ page, request }) => {
		// The intake webhook a Home Assistant automation or `curl` would call.
		expect(await deliverVoiceItem(request, "oat milk", 2)).toBe("pending");

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept oat milk" }).click();
		// The card goes as soon as the decision is recorded; leaving before that
		// would abort the request the click started.
		await expect(
			page.getByRole("button", { name: "Accept oat milk" }),
		).toHaveCount(0);

		await gotoList(page);
		const item = itemCard(page, "oat milk");
		await expect(item).toBeVisible();
		await expect(item).toContainText("Added via voice");
	});
});
