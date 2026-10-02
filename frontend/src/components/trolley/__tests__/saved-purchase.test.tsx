import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { orderFixture } from "#/components/analysis/__tests__/fixtures";
import { SavedPurchase } from "#/components/trolley/saved-purchase";
import { api, type TrolleyHandoff } from "#/lib/api";

vi.mock("sonner", () => ({ toast: vi.fn() }));

function handoff(status: TrolleyHandoff["status"]): TrolleyHandoff {
	return {
		id: "handoff-1",
		store: "woolworths",
		store_name: "Woolworths",
		status,
		created_at: "2026-10-02T00:00:00Z",
		expires_at: "2026-10-02T00:30:00Z",
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
	};
}

afterEach(() => vi.restoreAllMocks());

describe("SavedPurchase", () => {
	it("says the fill was saved as bought, and Undo takes it back", async () => {
		vi.spyOn(api.purchases, "listOrders").mockResolvedValue([
			orderFixture({ trolley_handoff_id: "other" }),
			orderFixture({ id: "mine" }),
		]);
		const undo = vi
			.spyOn(api.purchases, "undo")
			.mockResolvedValue({ items_restored: 2 });
		render(<SavedPurchase handoff={handoff("filled")} />);

		expect(
			await screen.findByText(/Saved as bought: 2 products, \$33\.20/),
		).toBeVisible();
		await userEvent.click(screen.getByRole("button", { name: "Undo" }));

		expect(undo).toHaveBeenCalledWith("mine");
		expect(
			await screen.findByText("Undone. These items are back on your list."),
		).toBeVisible();
	});

	it("does not look before the trolley is filled", () => {
		const list = vi.spyOn(api.purchases, "listOrders");
		render(<SavedPurchase handoff={handoff("claimed_by_store_tab")} />);
		expect(list).not.toHaveBeenCalled();
	});

	it("keeps looking until the shop is saved", async () => {
		vi.useFakeTimers({ shouldAdvanceTime: true });
		const list = vi
			.spyOn(api.purchases, "listOrders")
			.mockResolvedValueOnce([])
			.mockResolvedValue([orderFixture()]);
		render(<SavedPurchase handoff={handoff("filled_with_problems")} />);

		await vi.advanceTimersByTimeAsync(1600);

		expect(await screen.findByText(/Saved as bought/)).toBeVisible();
		expect(list).toHaveBeenCalledTimes(2);
		vi.useRealTimers();
	});
});
