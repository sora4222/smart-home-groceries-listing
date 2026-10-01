import { describe, expect, it } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { isPromoted, visibleProducts } from "#/lib/specials";

const plain = productFixture({ product_id: "plain" });
const special = productFixture({ product_id: "special", on_special: true });
const multibuy = productFixture({
	product_id: "multibuy",
	deals: [{ description: "2 for $5", min_quantity: 2, unit_price: "2.5" }],
});

describe("isPromoted", () => {
	it("counts specials and multibuys", () => {
		expect(isPromoted(plain)).toBe(false);
		expect(isPromoted(special)).toBe(true);
		expect(isPromoted(multibuy)).toBe(true);
	});
});

describe("visibleProducts", () => {
	it("shows everything, in order, unless specials only is on", () => {
		const all = [plain, special, multibuy];
		expect(visibleProducts(all, false)).toEqual(all);
		expect(visibleProducts(all, true)).toEqual([special, multibuy]);
	});
});
