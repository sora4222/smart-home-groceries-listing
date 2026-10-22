import { afterEach, describe, expect, it, vi } from "vitest";
import {
	buildFillProgram,
	FILL_PROGRESS_GLOBAL,
} from "#/lib/store-tab/bookmarklet";

const config = { apiBaseUrl: "https://grocery.test/", secret: "s3cret" };
/** The store page's globals, where the program keeps its progress. */
const page = window as unknown as Record<string, unknown>;
const progress = () => page[FILL_PROGRESS_GLOBAL];

/** Runs a desktop program as the store window does: as page script. */
function runProgram(fill: (...args: unknown[]) => unknown) {
	new Function(buildFillProgram(fill, config, {}, "report"))();
}

describe("the desktop fill program", () => {
	afterEach(() => {
		delete page[FILL_PROGRESS_GLOBAL];
		delete page.finishFill;
	});

	it("is running as soon as it starts, then reports the result message", async () => {
		// The program carries the fill's source text, so it reaches the test
		// through the page, as the store page would see it.
		runProgram(
			() =>
				new Promise((resolve) => {
					// biome-ignore lint/suspicious/noExplicitAny: page script, run from source text.
					(window as any).finishFill = resolve;
				}),
		);
		expect(progress()).toEqual({ state: "running" });

		const finish = page.finishFill as (v: unknown) => void;
		finish({ message: "Added 2 products.", checkout: { path: "/x" } });
		await vi.waitFor(() =>
			expect(progress()).toEqual({
				state: "completed",
				message: "Added 2 products.",
			}),
		);
	});

	it("reports a failure the person can read, instead of only logging it", async () => {
		const error = vi.spyOn(console, "error").mockImplementation(() => {});
		runProgram(() => Promise.reject(new Error("HTTP 401")));
		await vi.waitFor(() =>
			expect(progress()).toEqual({
				state: "failed",
				message: "Fill trolley failed: HTTP 401",
			}),
		);
		error.mockRestore();
	});

	it("never opens checkout or shows a dialog", async () => {
		const alert = vi.fn();
		const confirm = vi.fn();
		vi.stubGlobal("alert", alert);
		vi.stubGlobal("confirm", confirm);
		runProgram(async () => ({ message: "Done.", checkout: { path: "/c" } }));
		await vi.waitFor(() =>
			expect(progress()).toMatchObject({ state: "completed" }),
		);
		expect(alert).not.toHaveBeenCalled();
		expect(confirm).not.toHaveBeenCalled();
		vi.unstubAllGlobals();
	});
});
