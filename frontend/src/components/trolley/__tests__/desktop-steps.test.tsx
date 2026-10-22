import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { TrolleyHandoff } from "#/lib/api";

const { desktop } = vi.hoisted(() => ({
	desktop: {
		abilities: vi.fn(),
		openStore: vi.fn(),
		fillTrolley: vi.fn(),
		openCheckout: vi.fn(),
		notify: vi.fn(),
	},
}));
vi.mock("#/lib/desktop/bridge", () => ({ isDesktop: () => true }));
vi.mock("#/lib/desktop/commands", async (original) => ({
	...(await original<typeof import("#/lib/desktop/commands")>()),
	desktop,
}));

import { SendToWoolworths } from "#/components/trolley/send-to-woolworths";
import { api } from "#/lib/api";

function handoff(overrides: Partial<TrolleyHandoff> = {}): TrolleyHandoff {
	return {
		id: "h1",
		store: "woolworths",
		store_name: "Woolworths",
		status: "waiting_for_store_tab",
		created_at: "2026-10-02T00:00:00Z",
		expires_at: "2999-01-01T00:00:00Z",
		claimed_at: null,
		reported_at: null,
		delivery: {
			requested: { date: null, time_of_day: "any" },
			outcome: null,
			window_label: null,
			window_start: null,
			window_end: null,
			fee: null,
			problem: null,
		},
		lines: [],
		...overrides,
	} as TrolleyHandoff;
}

async function openSheet() {
	const user = userEvent.setup();
	render(
		<SendToWoolworths chosenCount={1}>
			<SendToWoolworths.Trigger />
			<SendToWoolworths.Content />
		</SendToWoolworths>,
	);
	await user.click(
		screen.getByRole("button", { name: /Send to Woolworths \(1 item\)/ }),
	);
	return user;
}

describe("SendToWoolworths in the desktop app", () => {
	beforeEach(() => {
		for (const command of Object.values(desktop)) command.mockReset();
		desktop.abilities.mockResolvedValue({
			version: "0.1.0",
			stores: [
				{
					store: "woolworths",
					login: true,
					fill_trolley: true,
					checkout: true,
				},
			],
		});
		for (const command of [
			desktop.openStore,
			desktop.fillTrolley,
			desktop.openCheckout,
			desktop.notify,
		])
			command.mockResolvedValue(undefined);
		vi.spyOn(api.trolleyHandoffs, "storeTabSecret").mockResolvedValue({
			secret: "s3cret",
		});
	});
	afterEach(() => vi.restoreAllMocks());

	it("fills the trolley in the app instead of using the bookmark", async () => {
		const create = vi
			.spyOn(api.trolleyHandoffs, "create")
			.mockResolvedValue(handoff());
		const open = vi.spyOn(window, "open");
		const user = await openSheet();

		const fill = await screen.findByRole("button", {
			name: "Fill trolley in the app",
		});
		expect(screen.queryByText("Fill Woolworths trolley")).toBeNull();
		await vi.waitFor(() => expect(fill).toBeEnabled());
		await user.click(fill);

		expect(create).toHaveBeenCalledWith("woolworths", {
			date: null,
			time_of_day: "any",
		});
		expect(desktop.fillTrolley).toHaveBeenCalledWith(
			"woolworths",
			expect.stringContaining('"secret":"s3cret"'),
		);
		expect(open).not.toHaveBeenCalled();
		expect(
			await screen.findByText(/Waiting for the Woolworths window/),
		).toBeInTheDocument();
		expect(screen.queryByText(/press the .* bookmark/)).toBeNull();
	});

	it("opens Woolworths to log in", async () => {
		const user = await openSheet();
		await user.click(
			await screen.findByRole("button", { name: "Log in to Woolworths" }),
		);
		expect(desktop.openStore).toHaveBeenCalledWith("woolworths");
	});

	it("offers checkout once the trolley is filled, and says so", async () => {
		vi.spyOn(api.trolleyHandoffs, "create").mockResolvedValue(
			handoff({ status: "filled" }),
		);
		const user = await openSheet();
		const fill = await screen.findByRole("button", {
			name: "Fill trolley in the app",
		});
		await vi.waitFor(() => expect(fill).toBeEnabled());
		await user.click(fill);

		await user.click(
			await screen.findByRole("button", { name: "Open checkout" }),
		);
		expect(desktop.openCheckout).toHaveBeenCalledWith("woolworths");
		expect(desktop.notify).toHaveBeenCalledWith(
			"Woolworths trolley filled",
			expect.any(String),
		);
	});

	it("shows the app's refusal", async () => {
		vi.spyOn(api.trolleyHandoffs, "create").mockResolvedValue(handoff());
		desktop.fillTrolley.mockRejectedValue(
			new Error("The store window could not be opened."),
		);
		const user = await openSheet();
		const fill = await screen.findByRole("button", {
			name: "Fill trolley in the app",
		});
		await vi.waitFor(() => expect(fill).toBeEnabled());
		await user.click(fill);
		expect(await screen.findByRole("alert")).toHaveTextContent(
			"The store window could not be opened.",
		);
	});
});
