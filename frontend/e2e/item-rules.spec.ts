import {
	API_BASE_URL,
	addItem,
	expect,
	gotoList,
	itemCard,
	ruleCard,
	test,
} from "./fixtures";
import type { Page } from "@playwright/test";

/**
 * Item rules in a real browser, against the real backend: make a rule on the
 * settings page, then watch it put chips on items as they reach the list.
 */

/** Fills in and submits the page's "New rule" form. */
async function addRule(
	page: Page,
	names: string[],
	filters: string[],
	applyToManual = false,
) {
	const form = page.getByRole("heading", { name: "New rule" }).locator("..");
	for (const name of names) {
		await form.getByLabel("Add an item name").fill(name);
		await form.getByLabel("Add an item name").press("Enter");
	}
	for (const filter of filters) {
		await form.getByLabel("Add a product filter").fill(filter);
		await form.getByLabel("Add a product filter").press("Enter");
	}
	if (applyToManual) {
		await form.getByRole("switch", { name: "Apply to manual additions" }).click();
	}
	await form.getByRole("button", { name: "Add rule" }).click();
	await expect(ruleCard(page, names[0])).toBeVisible();
}

test.describe("item rules page", () => {
	test("is reachable from the nav and starts empty", async ({ page }) => {
		await gotoList(page);
		await page.getByRole("link", { name: "Item Rules" }).click();

		await expect(page.getByRole("heading", { name: "Item Rules" })).toBeVisible();
		await expect(page.getByText("No rules yet.")).toBeVisible();
	});

	test("adds a rule and keeps it after a reload", async ({ page }) => {
		await gotoList(page, "/settings/item-rules");

		await addRule(page, ["toilet paper", "loo roll"], ["3 ply"]);

		await page.reload();
		const card = ruleCard(page, "toilet paper");
		await expect(card.getByLabel("Item names")).toContainText("loo roll");
		await expect(card.getByLabel("Product filters")).toContainText("3 ply");
		await expect(card).toContainText("Voice additions only");
	});

	test("edits a rule in place, including the manual toggle", async ({ page }) => {
		await gotoList(page, "/settings/item-rules");
		await addRule(page, ["milk"], ["a2"]);

		const card = ruleCard(page, "milk");
		await card.getByRole("button", { name: "Edit rule milk" }).click();
		await card.getByLabel("Remove filter a2").click();
		await card.getByLabel("Add a product filter").fill("full cream");
		await card.getByLabel("Add a product filter").press("Enter");
		await card.getByRole("switch", { name: "Apply to manual additions" }).click();
		await card.getByRole("button", { name: "Save rule" }).click();

		await expect(card.getByLabel("Product filters")).toHaveText("full cream");
		await expect(card).toContainText("Voice and manual additions");
	});

	test("deletes a rule after asking", async ({ page }) => {
		await gotoList(page, "/settings/item-rules");
		await addRule(page, ["milk"], ["a2"]);

		await page.getByRole("button", { name: "Delete rule milk" }).click();
		await page.getByRole("button", { name: "Confirm deleting rule milk" }).click();

		await expect(ruleCard(page, "milk")).toHaveCount(0);
		await expect(page.getByText("No rules yet.")).toBeVisible();
	});
});

test.describe("a rule reaching the list", () => {
	test("chips an accepted voice item", async ({ page, request }) => {
		await gotoList(page, "/settings/item-rules");
		await addRule(page, ["toilet paper"], ["3 ply"]);

		const delivered = await request.post(`${API_BASE_URL}/api/voice-requests`, {
			headers: { "x-webhook-secret": "e2e-webhook-secret" },
			data: { item: "Toilet Paper", quantity: 1 },
		});
		expect(delivered.status()).toBe(201);

		await gotoList(page, "/pending");
		await page.getByRole("button", { name: "Accept Toilet Paper" }).click();
		await expect(page.getByText("Item rules added the filter: 3 ply")).toBeVisible();

		await gotoList(page);
		const item = itemCard(page, "Toilet Paper");
		await expect(item.getByLabel("Product filters")).toHaveText("3 ply");

		// Taking the chip off the item leaves the rule alone.
		await item.getByLabel("Remove filter 3 ply").click();
		await expect(item.getByLabel("Product filters")).toHaveCount(0);
		await gotoList(page, "/settings/item-rules");
		await expect(
			ruleCard(page, "toilet paper").getByLabel("Product filters"),
		).toHaveText("3 ply");
	});

	test("leaves a typed item alone unless the rule opts in", async ({ page }) => {
		await gotoList(page, "/settings/item-rules");
		await addRule(page, ["milk"], ["a2"]);
		await addRule(page, ["bread"], ["wholemeal"], true);

		await gotoList(page);
		await addItem(page, "milk");
		await addItem(page, "bread");

		await expect(itemCard(page, "milk").getByLabel("Product filters")).toHaveCount(0);
		await expect(itemCard(page, "bread").getByLabel("Product filters")).toHaveText(
			"wholemeal",
		);
	});
});
