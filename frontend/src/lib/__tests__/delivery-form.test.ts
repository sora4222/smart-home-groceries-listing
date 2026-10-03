import { describe, expect, it } from "vitest";
import type { DeliverySettings } from "#/lib/api";
import { fromForm, parseAmount, toForm } from "#/lib/delivery-form";

const saved: DeliverySettings = {
	stores: [
		{
			store: "woolworths",
			store_name: "Woolworths",
			delivery_fee: "9",
			free_delivery_over: null,
			minimum_order: "50",
		},
		{
			store: "coles",
			store_name: "Coles",
			delivery_fee: null,
			free_delivery_over: null,
			minimum_order: null,
		},
	],
	mode: "minimise_delivery",
	max_delivery_spend: null,
};

describe("parseAmount", () => {
	it("reads empty as not set and allows a dollar sign", () => {
		expect(parseAmount("")).toEqual({ ok: true, value: null });
		expect(parseAmount("  ")).toEqual({ ok: true, value: null });
		expect(parseAmount("$9.5")).toEqual({ ok: true, value: "9.5" });
		expect(parseAmount("1000")).toEqual({ ok: true, value: "1000" });
	});

	it("refuses negatives, fractions of a cent, words and over $1000", () => {
		for (const input of ["-1", "9.999", "nine", "1000.01", "1,000"]) {
			expect(parseAmount(input)).toEqual({ ok: false });
		}
	});
});

describe("toForm and fromForm", () => {
	it("round-trips the saved settings", () => {
		const back = fromForm(toForm(saved));
		expect(back).toEqual({
			settings: {
				stores: saved.stores.map(({ store_name: _, ...rules }) => rules),
				mode: "minimise_delivery",
				max_delivery_spend: null,
			},
		});
	});

	it("names the store whose amount is wrong", () => {
		const form = toForm(saved);
		form.stores[1].deliveryFee = "abc";
		expect(fromForm(form)).toEqual({
			error: expect.stringContaining("Coles's amounts"),
		});
	});

	it("names the spending limit when it is wrong", () => {
		const form = toForm(saved);
		form.maxDeliverySpend = "-3";
		expect(fromForm(form)).toEqual({
			error: expect.stringContaining("delivery spending limit"),
		});
	});
});
