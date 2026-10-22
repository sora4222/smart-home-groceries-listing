import { expect, gotoList, test } from "./fixtures";

/**
 * The e2e suite runs with sign-in off (`VITE_AUTH_MODE=off` and the
 * backend's `DEV_AUTH_BYPASS=true`), because a real Clerk sign-in needs a
 * real Clerk account. These check that test mode says so plainly and never
 * strands anyone on a sign-in page. Real token checks are the backend's
 * `tests/clerk_tokens.rs` and `tests/auth_required.rs`.
 */
test.describe("sign-in off (test mode)", () => {
	test("says sign-in is off on every page", async ({ page }) => {
		await gotoList(page, "/");
		await expect(page.getByRole("status")).toHaveText(
			"Sign-in is off. Use this only on your own computer.",
		);
	});

	test("sends /sign-in to the grocery list", async ({ page }) => {
		await page.goto("/sign-in");
		await expect(page).toHaveURL(/\/$/);
		await expect(
			page.getByRole("navigation", { name: "Main navigation" }),
		).toBeVisible();
	});

	test("shows no account button", async ({ page }) => {
		await gotoList(page, "/");
		await expect(
			page.getByRole("button", { name: /open user menu/i }),
		).toHaveCount(0);
	});
});
