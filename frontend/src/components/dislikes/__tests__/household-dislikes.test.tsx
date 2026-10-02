import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { dislikeFixture } from "#/components/dislikes/__tests__/fixtures";
import { HouseholdDislikes } from "#/components/dislikes/household-dislikes";

describe("HouseholdDislikes", () => {
	it("shows each member's dislikes, mine first", () => {
		render(
			<HouseholdDislikes
				dislikes={[
					dislikeFixture(),
					dislikeFixture({ user_id: "me", user_name: "dev", mine: true }),
				]}
				onRemove={vi.fn()}
			/>,
		);

		const sections = screen.getAllByRole("region");
		expect(sections.map((s) => s.getAttribute("aria-label"))).toEqual([
			"Your dislikes",
			"Phu's dislikes",
		]);
		expect(
			within(sections[1]).getByText("Coles Full Cream Milk"),
		).toBeInTheDocument();
		expect(within(sections[1]).getByText("3L · Coles")).toBeInTheDocument();
	});

	it("lets me remove only my own dislike", async () => {
		const onRemove = vi.fn();
		render(
			<HouseholdDislikes
				dislikes={[
					dislikeFixture(),
					dislikeFixture({ user_id: "me", user_name: "dev", mine: true }),
				]}
				onRemove={onRemove}
			/>,
		);

		const removes = screen.getAllByRole("button", {
			name: /Remove my dislike/,
		});
		expect(removes).toHaveLength(1);
		await userEvent.click(removes[0]);
		expect(onRemove).toHaveBeenCalledWith({
			store: "coles",
			product_id: "c-milk-3l",
		});
	});

	it("says how to add one when there are none", () => {
		render(<HouseholdDislikes dislikes={[]} onRemove={vi.fn()} />);
		expect(
			screen.getByText(/Nobody dislikes a product yet/),
		).toBeInTheDocument();
	});
});
