import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { isDesktop } from "#/lib/desktop/bridge";
import { desktop } from "#/lib/desktop/commands";

type TauriWindow = Window & { __TAURI_INTERNALS__?: unknown };

describe("desktop bridge", () => {
	beforeEach(() => invoke.mockReset());
	afterEach(() => {
		delete (window as TauriWindow).__TAURI_INTERNALS__;
	});

	it("is off in a normal browser, and calls nothing", async () => {
		expect(isDesktop()).toBe(false);
		await expect(desktop.notify("Hi", "there")).rejects.toThrow(
			/only in the desktop app/,
		);
		expect(invoke).not.toHaveBeenCalled();
	});

	it("is on inside the desktop app", () => {
		(window as TauriWindow).__TAURI_INTERNALS__ = {};
		expect(isDesktop()).toBe(true);
	});

	it("calls the app's commands by their names", async () => {
		(window as TauriWindow).__TAURI_INTERNALS__ = {};
		invoke.mockResolvedValue(undefined);

		await desktop.openStore("woolworths");
		await desktop.fillTrolley("woolworths", "fill()");
		await desktop.openCheckout("woolworths");
		await desktop.notify("Trolley filled", "2 added.");

		expect(invoke.mock.calls).toEqual([
			["open_store", { store: "woolworths" }],
			["fill_store_trolley", { store: "woolworths", program: "fill()" }],
			["open_store_checkout", { store: "woolworths" }],
			["notify", { title: "Trolley filled", body: "2 added." }],
		]);
	});

	it("reads what the app can do at each store", async () => {
		(window as TauriWindow).__TAURI_INTERNALS__ = {};
		const abilities = {
			version: "0.1.0",
			stores: [
				{
					store: "woolworths",
					login: true,
					fill_trolley: true,
					checkout: true,
				},
				{ store: "coles", login: false, fill_trolley: false, checkout: false },
			],
		};
		invoke.mockResolvedValue(abilities);
		await expect(desktop.abilities()).resolves.toEqual(abilities);
		expect(invoke).toHaveBeenCalledWith("desktop_abilities", undefined);
	});
});
