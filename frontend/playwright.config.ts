import { defineConfig, devices } from "@playwright/test";

/**
 * An already-installed Chromium to use instead of Playwright's own download.
 * Set `E2E_CHROMIUM_PATH` on a machine where `npx playwright install` cannot
 * run; leave it unset and Playwright uses the browser it manages itself.
 */
const executablePath = process.env.E2E_CHROMIUM_PATH || undefined;

/**
 * End-to-end configuration. The stack has to be up first — `make up`, or
 * `pnpm dev` plus the backend — because these tests drive the real API and a
 * real PostgreSQL database rather than a mock.
 *
 * `API_BASE_URL` is where the tests reset state between runs; it defaults to
 * the same backend the app is pointed at.
 */
export default defineConfig({
	testDir: "./e2e",
	fullyParallel: false,
	workers: 1,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
	use: {
		baseURL: process.env.E2E_BASE_URL ?? "http://localhost:3000",
		trace: "retain-on-failure",
		screenshot: "only-on-failure",
	},
	projects: [
		{
			name: "desktop",
			use: { ...devices["Desktop Chrome"], launchOptions: { executablePath } },
		},
		{
			name: "mobile",
			use: { ...devices["Pixel 7"], launchOptions: { executablePath } },
		},
	],
});
