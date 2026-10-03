import type { APIRequestContext } from "@playwright/test";
import { API_BASE_URL, expect, gotoList, test } from "./fixtures";

/**
 * The order review screen (`/order`), in a real browser against the real
 * backend.
 *
 * The backend must run with `STORE_CLIENTS=fake`: prices come from the fake
 * catalogue (`backend/src/services/stores/fake.rs`), never a real store.
 */

/** Adds an item through the API and returns its id. */
async function addItem(
	request: APIRequestContext,
	name: string,
	quantity = 1,
): Promise<string> {
	const res = await request.post(`${API_BASE_URL}/api/grocery-items`, {
		data: { name, quantity },
	});
	expect(res.status()).toBe(201);
	return ((await res.json()) as { id: string }).id;
}

/** Chooses a product for an item through the API. */
async function choose(
	request: APIRequestContext,
	itemId: string,
	store: string,
	productId: string,
) {
	const res = await request.put(
		`${API_BASE_URL}/api/grocery-items/${itemId}/selection`,
		{ data: { store, product_id: productId } },
	);
	expect(res.status()).toBe(200);
}

/** Bananas at Woolworths, milk at Coles, bread with nothing chosen. */
async function commitAnOrder(request: APIRequestContext) {
	await choose(request, await addItem(request, "bananas", 5), "woolworths", "w-banana");
	await choose(request, await addItem(request, "milk"), "coles", "c-milk-3l");
	await addItem(request, "bread", 2);
	await request.post(`${API_BASE_URL}/api/grocery-items/commit`);
}

test.describe("order review", () => {
	test("with nothing committed it leads back to the list", async ({ page }) => {
		await gotoList(page, "/order");

		await expect(page.getByText(/Nothing is ready to order yet/)).toBeVisible();
		await page.getByRole("link", { name: "Go to the grocery list" }).click();
		await expect(page.getByRole("heading", { name: "Grocery List" })).toBeVisible();
	});

	test("the nav opens the order screen", async ({ page }) => {
		await gotoList(page);

		await page
			.getByRole("navigation", { name: "Main navigation" })
			.getByRole("link", { name: "Order" })
			.click();

		await expect(page.getByRole("heading", { name: "Order" })).toBeVisible();
	});

	test("committed items are priced and grouped by store", async ({
		page,
		request,
	}) => {
		await commitAnOrder(request);
		await gotoList(page, "/order");

		const woolworths = page.getByRole("region", { name: "Woolworths" });
		await expect(woolworths.getByRole("listitem", { name: "bananas" })).toContainText(
			"× 5",
		);
		await expect(woolworths).toContainText("Subtotal$4.00");
		await expect(
			woolworths.getByRole("button", { name: "Send to Woolworths (1 item)" }),
		).toBeEnabled();

		const coles = page.getByRole("region", { name: "Coles" });
		await expect(coles.getByRole("listitem", { name: "milk" })).toContainText(
			"Coles Full Cream Milk",
		);
		await expect(coles).toContainText("Subtotal$4.95");
		await coles.getByRole("button", { name: "Send to Coles (1 item)" }).click();
		const sheet = page.getByRole("dialog", { name: "Send to Coles" });
		await expect(sheet.getByRole("link", { name: "Fill Coles trolley" })).toBeVisible();
		await expect(sheet.getByText(/pick a delivery time on Coles/)).toBeVisible();
		await page.keyboard.press("Escape");

		const unchosen = page.getByRole("region", { name: "Items with no product" });
		await expect(unchosen).toContainText("bread");

		const total = page.getByRole("region", { name: "Order total" });
		await expect(total).toContainText("Total so far$8.95");
		await expect(total).toContainText("Delivery fees are not included");
	});

	test("removing the item with no product completes the total", async ({
		page,
		request,
	}) => {
		await commitAnOrder(request);
		await gotoList(page, "/order");
		await expect(page.getByRole("region", { name: "Order total" })).toContainText(
			"Total so far",
		);

		const list = await request.get(`${API_BASE_URL}/api/grocery-items`);
		const items = (await list.json()) as Array<{ id: string; name: string }>;
		const bread = items.find((item) => item.name === "bread");
		if (!bread) throw new Error("bread is not on the list");
		// The fake catalogue sells no bread, so remove it instead.
		await request.post(`${API_BASE_URL}/api/grocery-items/release`);
		await request.delete(`${API_BASE_URL}/api/grocery-items/${bread.id}`);
		await request.post(`${API_BASE_URL}/api/grocery-items/commit`);

		await page.getByRole("button", { name: "Check prices again" }).click();

		await expect(page.getByRole("region", { name: "Items with no product" })).toBeHidden();
		await expect(page.getByRole("region", { name: "Order total" })).toContainText(
			"Total for items$8.95",
		);
	});
});
