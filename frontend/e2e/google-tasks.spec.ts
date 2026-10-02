import type { APIRequestContext } from "@playwright/test";

import { API_BASE_URL, expect, gotoList, test } from "./fixtures";

/**
 * Google Tasks intake against the real backend running the fake Tasks
 * account (`GOOGLE_TASKS_CLIENT=fake`, `GOOGLE_TASKS_NO_BACKGROUND_POLL=true`,
 * `INTAKE_LLM_PROVIDER=fake`). The fake's sign-in skips Google and comes
 * straight back to `/settings/intake`, so the whole round trip runs here.
 * Tasks are added through the fake's development-only door, the way a person
 * would add them in the Tasks app.
 */

const TASKS = `${API_BASE_URL}/api/intake/google-tasks`;

/** Forgets any sign-in an earlier test left behind. */
async function disconnect(request: APIRequestContext) {
	await request.delete(`${TASKS}/sign-in`);
}

/** Connects the fake account and watches its Groceries list, by API. */
async function watchGroceries(request: APIRequestContext) {
	const started = await request.post(`${TASKS}/sign-in`);
	const { authorize_url } = (await started.json()) as { authorize_url: string };
	const state = new URL(authorize_url).searchParams.get("state");
	const finished = await request.post(`${TASKS}/sign-in/finish`, {
		data: { code: "fake-code", state },
	});
	expect(finished.status()).toBe(200);
	const saved = await request.put(TASKS, {
		data: { task_list_id: "fake-groceries", enabled: true, poll_seconds: 60 },
	});
	expect(saved.status()).toBe(200);
}

/** Adds a task to the fake Groceries list. */
async function addTask(request: APIRequestContext, title: string) {
	const added = await request.post(`${API_BASE_URL}/api/dev/google-tasks/tasks`, {
		data: { title },
	});
	expect(added.status()).toBe(201);
}

/** Waits until the triage step has put `name` in the Rejected tab. */
async function waitUntilRejected(request: APIRequestContext, name: string) {
	await expect
		.poll(async () => {
			const listed = await request.get(`${API_BASE_URL}/api/triage?tab=rejected`);
			const names = ((await listed.json()) as Array<{ parsed_name: string }>).map(
				(r) => r.parsed_name,
			);
			return names.includes(name);
		})
		.toBe(true);
}

test.describe("Google Tasks intake", () => {
	test("connects from the settings page and switches checking on", async ({
		page,
		request,
	}) => {
		await disconnect(request);

		await gotoList(page, "/settings/intake");
		await page.getByRole("button", { name: "Connect Google Tasks" }).click();

		// The fake "Google" sends the browser straight back with a code.
		await expect(
			page.getByText("Google Tasks connected. Now pick your grocery list."),
		).toBeVisible();
		await expect(page).toHaveURL(/\/settings\/intake$/);

		await page.getByRole("combobox", { name: "Grocery list" }).click();
		await page.getByRole("option", { name: "Groceries" }).click();
		await page.getByRole("switch", { name: "Check this list" }).click();
		await page.getByRole("button", { name: "Save" }).click();

		const card = page.getByTestId("google-tasks");
		await expect(page.getByText("Google Tasks settings saved")).toBeVisible();
		await expect(card.getByText("On", { exact: true })).toBeVisible();
		await expect(card.getByRole("button", { name: "Check now" })).toBeEnabled();
	});

	test("Check now brings a task to Pending Requests, never straight onto the list", async ({
		page,
		request,
	}) => {
		await watchGroceries(request);
		await addTask(request, "2 oat milk");

		await gotoList(page, "/settings/intake");
		await page.getByRole("button", { name: "Check now" }).click();
		await expect(page.getByText("1 new item found.")).toBeVisible();

		await gotoList(page, "/pending");
		await expect(page.getByRole("button", { name: "Accept oat milk" })).toBeVisible();
		const list = await request.get(`${API_BASE_URL}/api/grocery-items`);
		expect(await list.json()).toEqual([]);
	});

	test("a rejected task can be put back, and the next check skips the checker", async ({
		page,
		request,
	}) => {
		await watchGroceries(request);
		await addTask(request, "car service");
		await request.post(`${TASKS}/poll`);
		await waitUntilRejected(request, "car service");

		await gotoList(page, "/triage?tab=rejected");
		const card = page.locator('[data-request-name="car service"]');
		await expect(card).toContainText("Google Tasks");
		await page
			.getByRole("button", { name: "Put car service back in Google Tasks" })
			.click();
		await expect(card).toHaveCount(0);
		await expect(page.getByText(/car service put back in Google Tasks/)).toBeVisible();

		const polled = await request.post(`${TASKS}/poll`);
		expect(((await polled.json()) as { recorded: number }).recorded).toBe(1);
		await gotoList(page, "/pending");
		await expect(
			page.getByRole("button", { name: "Accept car service" }),
		).toBeVisible();
	});

	test("the nav has an Intake link to the settings page", async ({ page }) => {
		await gotoList(page);
		await page.getByRole("link", { name: "Intake" }).click();
		await expect(page.getByRole("heading", { name: "Intake" })).toBeVisible();
		await expect(page.getByText("Google Keep")).toBeVisible();
	});
});
