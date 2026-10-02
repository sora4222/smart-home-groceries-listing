import {
	API_BASE_URL,
	deliverVoiceItem,
	expect,
	gotoList,
	test,
} from "./fixtures";

/**
 * The Triage view against the real backend, which runs the keyword checker
 * (`INTAKE_LLM_PROVIDER=fake`): an item naming a service ("car service") is
 * rejected, everything else is approved. Held items need a checker that is
 * unsure, which the keyword checker never is, so the Held tab is covered by
 * the backend tests and the component tests.
 */

test.describe("the triage step", () => {
	test("sends a grocery item to Pending Requests, not to Triage", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "oat milk")).toBe("pending");

		await gotoList(page, "/triage");
		await expect(page.getByText("Nothing is held for review.")).toBeVisible();
		await page.getByRole("tab", { name: /Rejected/ }).click();
		await expect(page.getByText("Nothing was rejected.")).toBeVisible();
	});

	test("shows a rejected item with its reason, and Accept moves it to Pending Requests", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "car service")).toBe("rejected");

		await gotoList(page, "/triage?tab=rejected");
		const card = page.locator('[data-request-name="car service"]');
		await expect(card).toContainText("Sounds like a service");
		await expect(card).toContainText("Webhook");
		await expect(page.getByRole("tab", { name: "Rejected (1)" })).toBeVisible();

		await page.getByRole("button", { name: "Accept car service" }).click();
		await expect(card).toHaveCount(0);
		await expect(page.getByText("car service moved to Pending Requests")).toBeVisible();

		// Moved on, but nothing reaches the list until a person accepts it.
		await gotoList(page, "/pending");
		await expect(page.getByRole("button", { name: "Accept car service" })).toBeVisible();
		const list = await request.get(`${API_BASE_URL}/api/grocery-items`);
		expect(await list.json()).toEqual([]);
	});

	test("Reject confirms a rejected item, which then leaves the view", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "haircut")).toBe("rejected");

		await gotoList(page, "/triage?tab=rejected");
		await page.getByRole("button", { name: "Reject haircut" }).click();

		await expect(page.locator('[data-request-name="haircut"]')).toHaveCount(0);
		await gotoList(page, "/triage?tab=rejected");
		await expect(page.getByText("Nothing was rejected.")).toBeVisible();
	});

	test("the nav has a Triage link that opens the view", async ({ page }) => {
		await gotoList(page);
		await page.getByRole("link", { name: "Triage" }).click();
		await expect(page.getByRole("heading", { name: "Triage" })).toBeVisible();
	});
});
