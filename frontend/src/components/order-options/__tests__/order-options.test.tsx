import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { OrderOptions } from "#/components/order-options/order-options";
import type { OrderOption, OrderOptions as Plan } from "#/lib/api";

function option(overrides: Partial<OrderOption> = {}): OrderOption {
	return {
		kind: "woolworths",
		label: "All at Woolworths",
		recommended: true,
		is_current: false,
		stores: [
			{
				store: "woolworths",
				store_name: "Woolworths",
				lines: [],
				subtotal: "3.9",
				delivery_fee: "9",
				fee_known: true,
				free_delivery: false,
				below_minimum: false,
				minimum_order: null,
			},
		],
		missing: [],
		items_total: "3.9",
		delivery_total: "9",
		total: "12.9",
		complete: true,
		fees_known: true,
		meets_minimums: true,
		within_delivery_cap: true,
		picks: [{ grocery_item_id: "milk", store: "woolworths" }],
		...overrides,
	};
}

function plan(options: OrderOption[]): Plan {
	return {
		mode: "minimise_total",
		options,
		unchosen: [],
		exact: true,
		max_delivery_spend: null,
	};
}

describe("OrderOptions", () => {
	it("shows each option's total with delivery and marks the recommended one", () => {
		render(
			<OrderOptions
				plan={plan([
					option(),
					option({
						kind: "coles",
						label: "All at Coles",
						recommended: false,
						is_current: true,
						total: "14.78",
					}),
				])}
				onUse={vi.fn()}
			/>,
		);

		const best = screen.getByRole("region", { name: "All at Woolworths" });
		expect(best).toHaveTextContent("Recommended");
		expect(best).toHaveTextContent("$12.90");
		expect(best).toHaveTextContent("$3.90 + $9.00 delivery");
		const current = screen.getByRole("region", { name: "All at Coles" });
		expect(current).toHaveTextContent("Current");
		// The order buys this way already: nothing to switch to.
		expect(within(current).queryByRole("button")).not.toBeInTheDocument();
	});

	it("uses an option when asked", async () => {
		const onUse = vi.fn().mockResolvedValue(undefined);
		render(<OrderOptions plan={plan([option()])} onUse={onUse} />);

		await userEvent.click(
			screen.getByRole("button", { name: "Use All at Woolworths" }),
		);

		expect(onUse).toHaveBeenCalledWith(option());
	});

	it("says what an option leaves out and shows the fee hint", () => {
		render(
			<OrderOptions
				plan={plan([
					option({
						recommended: false,
						complete: false,
						fees_known: false,
						missing: [
							{
								grocery_item_id: "bread",
								name: "bread",
								reason: "No product chosen at Woolworths.",
							},
						],
					}),
				])}
				onUse={vi.fn()}
			>
				<p>Set delivery fees</p>
			</OrderOptions>,
		);

		expect(
			screen.getByText("bread: No product chosen at Woolworths."),
		).toBeInTheDocument();
		expect(screen.getByText("Leaves out 1 item.")).toBeInTheDocument();
		expect(screen.getByText("Set delivery fees")).toBeInTheDocument();
	});

	it("renders nothing with nothing to buy", () => {
		const { container } = render(
			<OrderOptions plan={plan([])} onUse={vi.fn()} />,
		);
		expect(container).toBeEmptyDOMElement();
	});
});
