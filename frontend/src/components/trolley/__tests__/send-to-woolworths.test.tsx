import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
	SendToWoolworths,
	WOOLWORTHS_TROLLEY_URL,
} from "#/components/trolley/send-to-woolworths";
import { api, type TrolleyHandoff } from "#/lib/api";

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
		lines: [
			{
				grocery_item_id: "i1",
				product_id: "88436",
				name: "Milk 2L",
				quantity: 2,
				outcome: null,
				problem: null,
			},
		],
		...overrides,
	};
}

function renderSheet(chosenCount = 1) {
	render(
		<SendToWoolworths chosenCount={chosenCount}>
			<SendToWoolworths.Trigger />
			<SendToWoolworths.Content />
		</SendToWoolworths>,
	);
}

describe("SendToWoolworths", () => {
	afterEach(() => vi.restoreAllMocks());

	it("cannot be opened before any Woolworths product is chosen", () => {
		renderSheet(0);
		expect(
			screen.getByRole("button", { name: /Send to Woolworths \(0 items\)/ }),
		).toBeDisabled();
	});

	it("builds the bookmark, sends the list and opens the trolley", async () => {
		vi.spyOn(api.trolleyHandoffs, "storeTabSecret").mockResolvedValue({
			secret: "s",
		});
		const create = vi
			.spyOn(api.trolleyHandoffs, "create")
			.mockResolvedValue(handoff());
		const open = vi.spyOn(window, "open").mockReturnValue(null);
		const user = userEvent.setup();
		renderSheet(1);

		await user.click(
			screen.getByRole("button", { name: /Send to Woolworths \(1 item\)/ }),
		);
		const bookmark = await screen.findByText("Fill Woolworths trolley");
		expect(bookmark.getAttribute("href")).toMatch(/^javascript:/);

		await user.click(
			screen.getByRole("button", { name: "Send and open Woolworths" }),
		);

		expect(open).toHaveBeenCalledWith(WOOLWORTHS_TROLLEY_URL, "_blank");
		// Tomorrow, any time, unless the household picks otherwise.
		expect(create).toHaveBeenCalledWith("woolworths", {
			date: null,
			time_of_day: "any",
		});
		expect(await screen.findByRole("status")).toHaveTextContent(
			"press the “Fill Woolworths trolley” bookmark",
		);
		expect(screen.getByText(/2 × Milk 2L/)).toBeInTheDocument();
	});

	it("sends the delivery day and time of day the household picked", async () => {
		vi.spyOn(api.trolleyHandoffs, "storeTabSecret").mockResolvedValue({
			secret: "s",
		});
		const create = vi
			.spyOn(api.trolleyHandoffs, "create")
			.mockResolvedValue(handoff());
		vi.spyOn(window, "open").mockReturnValue(null);
		const user = userEvent.setup();
		renderSheet(1);

		await user.click(
			screen.getByRole("button", { name: /Send to Woolworths/ }),
		);
		expect(screen.getByRole("button", { name: /^Tomorrow/ })).toHaveAttribute(
			"aria-pressed",
			"true",
		);
		const dayButtons = screen
			.getAllByRole("button", { pressed: false })
			.filter((b) => /\d+ \w{3}$/.test(b.textContent ?? ""));
		await user.click(dayButtons[0]);
		await user.click(screen.getByRole("button", { name: "Evening" }));
		await user.click(
			screen.getByRole("button", { name: "Send and open Woolworths" }),
		);

		const [, delivery] = create.mock.calls[0];
		expect(delivery.time_of_day).toBe("evening");
		expect(delivery.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
	});

	it("shows the reserved delivery time once the bookmark reports", async () => {
		const { HandoffStatus } = await import(
			"#/components/trolley/handoff-status"
		);
		render(
			<HandoffStatus
				handoff={handoff({
					status: "filled",
					delivery: {
						requested: { date: null, time_of_day: "any" },
						outcome: "reserved",
						window_label: "7:00am - 10:00am",
						window_start: "2026-10-03T07:00:00",
						window_end: "2026-10-03T10:00:00",
						fee: "15",
						problem: null,
					},
				})}
			/>,
		);

		expect(
			screen.getByText(/Delivery: Sat,? 3 Oct, 7:00am - 10:00am \(\$15\.00\)/),
		).toBeInTheDocument();
		expect(screen.getByText(/change it on Woolworths/)).toBeInTheDocument();
	});

	it("shows the backend's reason when nothing can be sent", async () => {
		vi.spyOn(api.trolleyHandoffs, "storeTabSecret").mockResolvedValue({
			secret: "s",
		});
		const { ApiError } = await import("#/lib/api");
		vi.spyOn(api.trolleyHandoffs, "create").mockRejectedValue(
			new ApiError(422, {
				detail: "No item on the list has a Woolworths product chosen yet.",
			}),
		);
		const close = vi.fn();
		vi.spyOn(window, "open").mockReturnValue({ close } as unknown as Window);
		const user = userEvent.setup();
		renderSheet(1);

		await user.click(
			screen.getByRole("button", { name: /Send to Woolworths/ }),
		);
		await user.click(
			screen.getByRole("button", { name: "Send and open Woolworths" }),
		);

		expect(await screen.findByRole("alert")).toHaveTextContent(
			"No item on the list has a Woolworths product",
		);
		expect(close).toHaveBeenCalled();
	});
});
