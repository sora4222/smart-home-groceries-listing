import type { APIRequestContext } from "@playwright/test";
import { API_BASE_URL, expect, gotoList, test } from "./fixtures";

/**
 * Purchase history and the Spending page (`/analysis`), in a real browser
 * against the real backend with `STORE_CLIENTS=fake`.
 *
 * A shop is saved as bought when the "Fill Woolworths trolley" bookmark
 * reports. The bookmark runs on woolworths.com.au, so these tests make its
 * two calls (claim, report) through the API, exactly as it sends them.
 */

/** Adds an item, chooses a Woolworths product for it, and returns its id. */
async function chosenAtWoolworths(
	request: APIRequestContext,
	name: string,
	quantity: number,
	productId: string,
): Promise<string> {
	const added = await request.post(`${API_BASE_URL}/api/grocery-items`, {
		data: { name, quantity },
	});
	expect(added.status()).toBe(201);
	const { id } = (await added.json()) as { id: string };
	const chosen = await request.put(
		`${API_BASE_URL}/api/grocery-items/${id}/selection`,
		{ data: { store: "woolworths", product_id: productId } },
	);
	expect(chosen.status()).toBe(200);
	return id;
}

/**
 * Sends the Woolworths choices to the trolley and reports every product
 * added with a $15 delivery window — what the bookmark does — then waits
 * for the shop to be saved.
 */
async function fillTrolley(request: APIRequestContext, productIds: string[]) {
	const { secret } = (await (
		await request.get(`${API_BASE_URL}/api/trolley-handoffs/store-tab-secret`)
	).json()) as { secret: string };
	await request.post(`${API_BASE_URL}/api/trolley-handoffs`, {
		data: { store: "woolworths" },
	});
	const claimed = await request.post(
		`${API_BASE_URL}/api/store-tab/trolley-handoffs/claim`,
		{ data: JSON.stringify({ secret, store: "woolworths" }) },
	);
	const { handoff_id } = (await claimed.json()) as { handoff_id: string };
	const reported = await request.post(
		`${API_BASE_URL}/api/store-tab/trolley-handoffs/${handoff_id}/report`,
		{
			data: JSON.stringify({
				secret,
				lines: productIds.map((product_id) => ({ product_id, outcome: "added" })),
				delivery: {
					outcome: "reserved",
					window_label: "7:00am - 10:00am",
					window_start: "2026-10-03T07:00:00",
					window_end: "2026-10-03T10:00:00",
					fee: "15",
				},
			}),
		},
	);
	expect(reported.status()).toBe(200);
	await expect
		.poll(async () => {
			const shops = await request.get(`${API_BASE_URL}/api/purchase-orders`);
			return ((await shops.json()) as unknown[]).length;
		}, { message: "the filled trolley should be saved as bought" })
		.toBe(1);
}

test.describe("spending", () => {
	test("with nothing bought the page says how a shop gets there", async ({
		page,
	}) => {
		await gotoList(page, "/analysis");

		const latest = page.getByRole("region", { name: "Latest shop" });
		await expect(latest).toContainText("Nothing yet");
		await expect(page.getByText("Nothing bought in this range.")).toBeVisible();
	});

	test("a filled trolley shows on the Spending page, by every view", async ({
		page,
		request,
	}) => {
		await chosenAtWoolworths(request, "milk", 2, "w-milk-2l");
		await chosenAtWoolworths(request, "bananas", 5, "w-banana");
		await fillTrolley(request, ["w-milk-2l", "w-banana"]);

		await gotoList(page);
		await page
			.getByRole("navigation", { name: "Main navigation" })
			.getByRole("link", { name: "Spending" })
			.click();

		const latest = page.getByRole("region", { name: "Latest shop" });
		await expect(latest).toContainText("Woolworths");
		await expect(latest).toContainText("delivery $15.00");
		await expect(page.getByRole("img", { name: /Spend per month/ })).toBeVisible();

		await page.getByRole("tab", { name: "By item" }).click();
		const items = page.getByRole("list", { name: "Spend by item" });
		await expect(items).toContainText("milk");
		await expect(items).toContainText("bananas");
		await page.getByRole("button", { name: "Show prices paid for milk" }).click();
		await expect(
			page.getByRole("table", { name: "Price paid for milk, oldest first" }),
		).toContainText("$3.10 × 2");

		await page.getByRole("tab", { name: "By store" }).click();
		await expect(page.getByRole("list", { name: "Spend by store" })).toContainText(
			"delivery $15.00",
		);

		await page.getByRole("tab", { name: "By category" }).click();
		const categories = page.getByRole("list", { name: "Spend by category" });
		await expect(categories).toContainText("Dairy & eggs");
		await expect(categories).toContainText("Fruit & veg");
	});

	test("filters narrow the views and stay in the address", async ({
		page,
		request,
	}) => {
		await chosenAtWoolworths(request, "milk", 1, "w-milk-2l");
		await chosenAtWoolworths(request, "bananas", 1, "w-banana");
		await fillTrolley(request, ["w-milk-2l", "w-banana"]);

		await gotoList(page, "/analysis?view=item");
		await page.getByLabel("Category", { exact: true }).selectOption("Fruit & veg");

		await expect(page).toHaveURL(/category=Fruit/);
		const items = page.getByRole("list", { name: "Spend by item" });
		await expect(items).toContainText("bananas");
		await expect(items).not.toContainText("milk");

		await page.getByLabel("Store", { exact: true }).selectOption("coles");
		await expect(page.getByText("Nothing bought in this range.")).toBeVisible();
	});

	test("bought items leave the list, and Undo brings them back", async ({
		page,
		request,
	}) => {
		await chosenAtWoolworths(request, "milk", 1, "w-milk-2l");
		await fillTrolley(request, ["w-milk-2l"]);

		await gotoList(page);
		await expect(page.locator('[data-item-name="milk"]')).toHaveCount(0);

		await gotoList(page, "/analysis");
		await page.getByRole("button", { name: /Undo the Woolworths shop/ }).click();
		await expect(page.getByText(/Undone: the Woolworths shop/)).toBeVisible();
		await expect(
			page.getByRole("region", { name: "Latest shop" }),
		).toContainText("Nothing yet");

		await gotoList(page);
		await expect(page.locator('[data-item-name="milk"]')).toHaveCount(1);
	});

	test("the price comparison says how often a product was bought", async ({
		page,
		request,
	}) => {
		await chosenAtWoolworths(request, "milk", 2, "w-milk-2l");
		await fillTrolley(request, ["w-milk-2l"]);
		await request.post(`${API_BASE_URL}/api/grocery-items`, {
			data: { name: "milk", quantity: 1 },
		});

		await gotoList(page);
		await page.getByRole("button", { name: "Compare prices for milk" }).click();
		const bought = page
			.getByRole("dialog")
			.getByRole("button", { name: /Bought Full Cream Milk once/ });
		await expect(bought).toHaveCount(1);
		await bought.click();
		await expect(page.getByRole("dialog")).toContainText("$3.10 each × 2");
	});
});
