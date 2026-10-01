import { describe, expect, it } from "vitest";

import { formatMoney, formatUnitAmount, formatUnitPrice } from "#/lib/money";

describe("formatMoney", () => {
	it("shows dollars and cents", () => {
		expect(formatMoney("4.5")).toBe("$4.50");
		expect(formatMoney("4")).toBe("$4.00");
	});

	it("passes an unreadable value through unchanged", () => {
		expect(formatMoney("n/a")).toBe("n/a");
	});
});

describe("formatUnitAmount", () => {
	it("uses cents for a dollar or more", () => {
		expect(formatUnitAmount("1.655")).toBe("$1.66");
	});

	it("keeps a third decimal under a dollar", () => {
		expect(formatUnitAmount("0.155")).toBe("$0.155");
		expect(formatUnitAmount("0.32")).toBe("$0.32");
	});

	it("keeps a fourth decimal under a cent", () => {
		expect(formatUnitAmount("0.0028")).toBe("$0.0028");
	});
});

describe("formatUnitPrice", () => {
	it("names the basis", () => {
		expect(
			formatUnitPrice({ amount: "0.49", per: "100g", converted_from: "1kg" }),
		).toBe("$0.49 / 100g");
	});
});
