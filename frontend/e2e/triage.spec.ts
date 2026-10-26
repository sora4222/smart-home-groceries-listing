import {
	API_BASE_URL,
	deliverVoiceItem,
	expect,
	gotoList,
	test,
} from "./fixtures";

/**
 * Triage, the lower half of Pending, against the real backend's keyword
 * checker (`INTAKE_LLM_PROVIDER=fake`): an item naming a service ("car
 * service") is rejected, everything else is approved. Held items need a
 * checker that is unsure, which the keyword checker never is, so the Held tab
 * is covered by the backend and component tests.
 */

test.describe("the triage step", () => {
	test("sends a grocery item to the pending list, not to Triage", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "oat milk")).toBe("pending");

		await gotoList(page, "/pending");
		await expect(page.getByRole("button", { name: "Accept oat milk" })).toBeVisible();
		await expect(page.getByText("Nothing held for review.")).toBeVisible();
		await page.getByRole("tab", { name: /Rejected/ }).click();
		await expect(page.getByText("Nothing rejected.")).toBeVisible();
	});

	test("shows a rejected item under the pending list, and Accept moves it up", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "car service")).toBe("rejected");

		await gotoList(page, "/pending?tab=rejected");
		await expect(page.getByText("No items pending")).toBeVisible();
		const card = page.locator('[data-request-name="car service"]');
		await expect(card).toContainText("Sounds like a service");
		await expect(card).toContainText("Webhook");
		await expect(page.getByRole("tab", { name: "Rejected (1)" })).toBeVisible();

		await card.getByRole("button", { name: "Accept car service" }).click();
		await expect(card).toHaveCount(0);

		// Moved up, but nothing reaches the list until a person accepts it.
		const pending = page.getByRole("region", { name: "Pending requests" });
		await expect(
			pending.getByRole("button", { name: "Accept car service" }),
		).toBeVisible();
		const list = await request.get(`${API_BASE_URL}/api/grocery-items`);
		expect(await list.json()).toEqual([]);
	});

	test("Reject confirms a rejected item, which then leaves the page", async ({
		page,
		request,
	}) => {
		expect(await deliverVoiceItem(request, "haircut")).toBe("rejected");

		await gotoList(page, "/pending?tab=rejected");
		await page.getByRole("button", { name: "Reject haircut" }).click();

		await expect(page.locator('[data-request-name="haircut"]')).toHaveCount(0);
		await gotoList(page, "/pending?tab=rejected");
		await expect(page.getByText("Nothing rejected.")).toBeVisible();
	});

});
