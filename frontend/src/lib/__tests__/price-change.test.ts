import { describe, expect, it } from "vitest";

import { priceChangeText } from "#/lib/price-change";

describe("priceChangeText", () => {
	it("says the price went up, and from what", () => {
		expect(priceChangeText({ price_change: "up", chosen_price: "2.8" })).toBe(
			"Up from $2.80",
		);
	});

	it("says the price went down, and from what", () => {
		expect(
			priceChangeText({ price_change: "down", chosen_price: "5.20" }),
		).toBe("Down from $5.20");
	});

	it("says nothing when the price is the same", () => {
		expect(
			priceChangeText({ price_change: "same", chosen_price: "4.95" }),
		).toBeNull();
	});

	it("says nothing when either price is unknown", () => {
		expect(priceChangeText({ price_change: null, chosen_price: "4.95" })).toBe(
			null,
		);
		expect(priceChangeText({ price_change: "up", chosen_price: null })).toBe(
			null,
		);
	});
});
