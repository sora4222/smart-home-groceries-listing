import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { notify, desktopState } = vi.hoisted(() => ({
	notify: vi.fn(),
	desktopState: { on: true },
}));
vi.mock("#/lib/desktop/bridge", () => ({ isDesktop: () => desktopState.on }));
vi.mock("#/lib/desktop/commands", () => ({ desktop: { notify } }));

import { notifyDesktop } from "#/lib/desktop/notify";

const notice = { title: "Title", body: "Body" };

describe("notifyDesktop", () => {
	beforeEach(() => {
		notify.mockReset().mockResolvedValue(undefined);
		desktopState.on = true;
	});
	afterEach(() => vi.restoreAllMocks());

	it("sends the notice in the desktop app", () => {
		notifyDesktop(notice);
		expect(notify).toHaveBeenCalledWith("Title", "Body");
	});

	it("does nothing in a browser", () => {
		desktopState.on = false;
		notifyDesktop(notice);
		expect(notify).not.toHaveBeenCalled();
	});

	it("warns instead of failing when the app refuses", async () => {
		const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
		notify.mockRejectedValue(new Error("refused"));
		notifyDesktop(notice);
		await vi.waitFor(() =>
			expect(warn).toHaveBeenCalledWith(
				"[desktop] notice failed",
				expect.any(Error),
			),
		);
	});
});
