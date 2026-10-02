import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { orderFixture } from "#/components/analysis/__tests__/fixtures";
import { LatestOrderCard } from "#/components/analysis/latest-order-card";
import { RecentShops } from "#/components/analysis/recent-shops";
import { ApiError, api } from "#/lib/api";

vi.mock("sonner", () => ({ toast: vi.fn() }));

afterEach(() => vi.restoreAllMocks());

describe("RecentShops", () => {
	it("undoes a shop and re-reads the page", async () => {
		const undo = vi
			.spyOn(api.purchases, "undo")
			.mockResolvedValue({ items_restored: 2 });
		const onChanged = vi.fn();
		render(<RecentShops orders={[orderFixture()]} onChanged={onChanged} />);

		await userEvent.click(
			screen.getByRole("button", { name: /Undo the Woolworths shop/ }),
		);

		expect(undo).toHaveBeenCalledWith("order-1");
		expect(onChanged).toHaveBeenCalled();
	});

	it("treats a shop undone elsewhere as undone", async () => {
		vi.spyOn(api.purchases, "undo").mockRejectedValue(
			new ApiError(404, { detail: "gone" }),
		);
		const onChanged = vi.fn();
		render(<RecentShops orders={[orderFixture()]} onChanged={onChanged} />);

		await userEvent.click(screen.getByRole("button", { name: /Undo/ }));

		expect(onChanged).toHaveBeenCalled();
	});

	it("keeps the shop when Undo fails", async () => {
		vi.spyOn(api.purchases, "undo").mockRejectedValue(new Error("offline"));
		vi.spyOn(console, "error").mockImplementation(() => {});
		const onChanged = vi.fn();
		render(<RecentShops orders={[orderFixture()]} onChanged={onChanged} />);

		await userEvent.click(screen.getByRole("button", { name: /Undo/ }));

		expect(onChanged).not.toHaveBeenCalled();
	});

	it("shows nothing when no shop is saved", () => {
		const { container } = render(
			<RecentShops orders={[]} onChanged={vi.fn()} />,
		);
		expect(container).toBeEmptyDOMElement();
	});
});

describe("LatestOrderCard", () => {
	it("shows the newest shop's total", () => {
		render(<LatestOrderCard order={orderFixture()} />);
		const card = screen.getByRole("region", { name: "Latest shop" });
		expect(card).toHaveTextContent("$33.20");
		expect(card).toHaveTextContent("items $18.20 + delivery $15.00");
	});

	it("says how a shop gets here when there is none", () => {
		render(<LatestOrderCard order={null} />);
		expect(screen.getByText("Nothing yet")).toBeVisible();
	});
});
