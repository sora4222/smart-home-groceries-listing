import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DeliverySettingsForm } from "#/components/delivery/delivery-settings-form";
import type { DeliverySettings } from "#/lib/api";

const unset: DeliverySettings = {
	stores: [
		{
			store: "woolworths",
			store_name: "Woolworths",
			delivery_fee: null,
			free_delivery_over: null,
			minimum_order: null,
		},
		{
			store: "coles",
			store_name: "Coles",
			delivery_fee: "10",
			free_delivery_over: null,
			minimum_order: null,
		},
	],
	mode: "minimise_total",
	max_delivery_spend: null,
};

function renderForm(onSave = vi.fn().mockResolvedValue(undefined)) {
	render(<DeliverySettingsForm settings={unset} onSave={onSave} />);
	return onSave;
}

describe("DeliverySettingsForm", () => {
	it("shows each store's saved amounts and the chosen mode", () => {
		renderForm();

		const coles = screen.getByRole("group", { name: "Coles" });
		expect(coles).toBeInTheDocument();
		expect(screen.getAllByLabelText("Delivery fee")[1]).toHaveValue("10");
		expect(screen.getByRole("radio", { name: "Cheapest total" })).toBeChecked();
	});

	it("saves typed amounts and a new mode", async () => {
		const onSave = renderForm();

		await userEvent.type(screen.getAllByLabelText("Delivery fee")[0], "9");
		await userEvent.type(
			screen.getAllByLabelText("Free delivery from")[0],
			"$250",
		);
		await userEvent.click(screen.getByRole("radio", { name: "Coles only" }));
		await userEvent.type(
			screen.getByLabelText("Most to spend on delivery"),
			"15",
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith({
			stores: [
				{
					store: "woolworths",
					delivery_fee: "9",
					free_delivery_over: "250",
					minimum_order: null,
				},
				{
					store: "coles",
					delivery_fee: "10",
					free_delivery_over: null,
					minimum_order: null,
				},
			],
			mode: "coles_only",
			max_delivery_spend: "15",
		});
	});

	it("explains a wrong amount and sends nothing", async () => {
		const onSave = renderForm();

		await userEvent.type(screen.getAllByLabelText("Minimum order")[0], "lots");
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(screen.getByRole("alert")).toHaveTextContent("Woolworths");
		expect(onSave).not.toHaveBeenCalled();
	});
});
