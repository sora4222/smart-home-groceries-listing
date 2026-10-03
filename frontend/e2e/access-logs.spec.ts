import { API_BASE_URL, expect, gotoList, test } from "./fixtures";

/**
 * The `/logs` page against the real backend: a request made by anyone shows
 * up with its path and status, and health checks stay hidden until asked
 * for. The backend writes each row just after it answers, so the tests wait
 * for the row through the API before opening the page.
 */

test.describe("the access log", () => {
	test("shows a request that was just made, with its status", async ({
		page,
		request,
	}) => {
		const path = `/api/not-a-page-${Date.now()}`;
		expect((await request.get(`${API_BASE_URL}${path}`)).status()).toBe(404);
		await expect
			.poll(async () => {
				const listed = await request.get(`${API_BASE_URL}/api/access-logs`);
				const { entries } = (await listed.json()) as {
					entries: Array<{ path: string }>;
				};
				return entries.some((e) => e.path === path);
			}, { message: `${path} should be logged` })
			.toBe(true);

		await gotoList(page, "/logs");

		const row = page.locator(`[data-log-path="${path}"]`);
		await expect(row).toContainText("404");
		await expect(row).toContainText("GET");
	});

	test("hides health checks until the switch is turned on", async ({
		page,
		request,
	}) => {
		await request.get(`${API_BASE_URL}/api/health`);
		await expect
			.poll(async () => {
				const listed = await request.get(
					`${API_BASE_URL}/api/access-logs?hide_health_checks=false`,
				);
				const { entries } = (await listed.json()) as {
					entries: Array<{ path: string }>;
				};
				return entries.some((e) => e.path === "/api/health");
			}, { message: "the health check should be logged" })
			.toBe(true);

		await gotoList(page, "/logs");
		const health = page.locator('[data-log-path="/api/health"]');
		await expect(page.getByRole("list", { name: "Access log" })).toBeVisible();
		await expect(health).toHaveCount(0);

		await page.getByLabel("Show health checks").click();
		await expect(health.first()).toBeVisible();
	});

	test("is reachable from the nav", async ({ page }) => {
		await gotoList(page, "/");
		await page.getByRole("link", { name: "Logs" }).click();
		await expect(page.getByRole("heading", { name: "Access log" })).toBeVisible();
	});
});
