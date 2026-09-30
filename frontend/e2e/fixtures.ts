import { type Page, expect, test as base } from "@playwright/test";

/** Where the backend answers. The app itself is reached through `baseURL`. */
export const API_BASE_URL = process.env.E2E_API_BASE_URL ?? "http://localhost:8000";

interface GroceryFixtures {
	/** A page that fails the test if the browser logged an error. */
	page: Page;
}

/**
 * The shared test setup.
 *
 * Every test starts from an empty list, an empty confirmation queue and no
 * item rules, and
 * every test fails if the browser threw, logged an error, or got a 5xx from the
 * backend — a feature that "works" while React complains in the console is not
 * working.
 *
 * Chrome logs "Failed to load resource" for any 4xx, and this app provokes 4xx
 * deliberately (the duplicate-item 409 is a feature). Those lines are therefore
 * left out and the status codes are judged separately: a 5xx is always a bug,
 * whatever the test was doing.
 */
export const test = base.extend<GroceryFixtures>({
	page: async ({ page, request }, use) => {
		// Release anything a previous run committed, then clear the list.
		await request.post(`${API_BASE_URL}/api/grocery-items/release`);
		const existing = await request.get(`${API_BASE_URL}/api/grocery-items`);
		for (const item of (await existing.json()) as Array<{ id: string }>) {
			await request.delete(`${API_BASE_URL}/api/grocery-items/${item.id}`);
		}
		// Clear the confirmation queue too, so a nav badge or a leftover card
		// from an earlier run cannot change what a test sees.
		const pending = await request.get(`${API_BASE_URL}/api/voice-requests`);
		for (const item of (await pending.json()) as Array<{ id: string }>) {
			await request.post(`${API_BASE_URL}/api/voice-requests/${item.id}/reject`);
		}
		// Rules would otherwise put chips on items other tests add.
		const rules = await request.get(`${API_BASE_URL}/api/item-rules`);
		for (const rule of (await rules.json()) as Array<{ id: string }>) {
			await request.delete(`${API_BASE_URL}/api/item-rules/${rule.id}`);
		}

		const problems: string[] = [];
		page.on("console", (message) => {
			if (message.type() !== "error") return;
			const text = message.text();
			// The HTTP status behind this is asserted below; a deliberate 409 is
			// not a defect.
			if (text.startsWith("Failed to load resource")) return;
			problems.push(`console: ${text}`);
		});
		page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));
		page.on("response", (response) => {
			if (response.status() >= 500) {
				problems.push(`server error: ${response.status()} ${response.url()}`);
			}
		});

		await use(page);

		expect(problems, "the browser reported no errors").toEqual([]);
	},
});

export { expect };

/**
 * Navigates and waits for React to hydrate.
 *
 * The app is server-rendered, so the markup is on screen before any handler is
 * attached; typing or clicking before that point is silently lost. `#app`
 * carries `data-hydrated` once the root has mounted, which is the signal to
 * start interacting.
 */
export async function gotoList(page: Page, path = "/") {
	await page.goto(path);
	await expect(page.locator('#app[data-hydrated="true"]')).toBeAttached();
}

/**
 * The add-item form's fields.
 *
 * Exact labels throughout: Playwright matches a label by substring by default,
 * and "Item" would also match the form's own "Add a grocery item" while
 * "Quantity" would match a "Quantity 2" badge.
 */
export function addForm(page: Page) {
	return {
		name: page.getByLabel("Item", { exact: true }),
		quantity: page.getByLabel("Quantity", { exact: true }),
		submit: page.getByRole("button", { name: "Add item" }),
	};
}

/**
 * The card for one named item on the list.
 *
 * Addressed by `data-item-name` rather than by its text: with the editor open
 * the name lives in an input's value, which text filtering cannot see.
 */
export function itemCard(page: Page, name: string) {
	return page.locator(
		`[data-testid="grocery-item"][data-item-name="${name}"]`,
	);
}

/** Adds an item through the form and waits for it to appear on the list. */
export async function addItem(page: Page, name: string, quantity?: number) {
	const form = addForm(page);
	await form.name.fill(name);
	if (quantity !== undefined) {
		await form.quantity.fill(String(quantity));
	}
	await form.submit.click();
	await expect(itemCard(page, name)).toBeVisible();
}

/**
 * The card for one rule on the Item Rules page, addressed by its first item
 * name (`data-rule-name`) — with the editor open the names are chips in a form,
 * not text in the card.
 */
export function ruleCard(page: Page, name: string) {
	return page.locator(`[data-testid="item-rule"][data-rule-name="${name}"]`);
}
