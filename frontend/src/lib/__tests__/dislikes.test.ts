import { describe, expect, it } from "vitest";

import type { DislikeOverride, ProductDislike } from "#/lib/api";
import {
	dislikesOf,
	dislikeWarning,
	groupByMember,
	isOverridden,
} from "#/lib/dislikes";

function dislike(overrides: Partial<ProductDislike> = {}): ProductDislike {
	return {
		store: "coles",
		store_name: "Coles",
		product_id: "c-milk-3l",
		name: "Full Cream Milk",
		brand: "Coles",
		package_size: "3L",
		user_id: "phu-id",
		user_name: "Phu",
		mine: false,
		disliked_at: "2026-10-01T10:00:00Z",
		...overrides,
	};
}

const milk = { store: "coles", product_id: "c-milk-3l" } as const;

describe("dislikesOf", () => {
	it("finds the product's dislikes, oldest first", () => {
		const later = dislike({
			user_name: "Jesse",
			disliked_at: "2026-10-02T00:00:00Z",
		});
		const earlier = dislike();
		expect(dislikesOf([later, earlier], milk)).toEqual([earlier, later]);
	});

	it("does not count the same id at the other store", () => {
		expect(dislikesOf([dislike({ store: "woolworths" })], milk)).toEqual([]);
	});
});

describe("isOverridden", () => {
	const override: DislikeOverride = {
		grocery_item_id: "item-1",
		store: "coles",
		product_id: "c-milk-3l",
		overridden_by: "dev-user",
		overridden_at: "2026-10-01T10:00:00Z",
	};

	it("is true only for the same item and product", () => {
		expect(isOverridden([override], "item-1", milk)).toBe(true);
		expect(isOverridden([override], "item-2", milk)).toBe(false);
		expect(
			isOverridden([override], "item-1", { ...milk, store: "woolworths" }),
		).toBe(false);
	});
});

describe("dislikeWarning", () => {
	it("names one member", () => {
		expect(dislikeWarning([dislike()])).toBe(
			"Phu disliked this item previously.",
		);
	});

	it("says You for the caller's own dislike", () => {
		expect(dislikeWarning([dislike({ mine: true })])).toBe(
			"You disliked this item previously.",
		);
	});

	it("joins several names", () => {
		expect(
			dislikeWarning([
				dislike(),
				dislike({ user_name: "Jesse" }),
				dislike({ mine: true }),
			]),
		).toBe("Phu, Jesse and You disliked this item previously.");
	});
});

describe("groupByMember", () => {
	it("puts my dislikes first, then others by name", () => {
		const groups = groupByMember([
			dislike({ user_id: "z", user_name: "Zed", product_id: "1" }),
			dislike({ product_id: "2" }),
			dislike({ user_id: "me", user_name: "dev", mine: true }),
			dislike({ product_id: "3" }),
		]);

		expect(groups.map((g) => g.userName)).toEqual(["dev", "Phu", "Zed"]);
		expect(groups[1].dislikes.map((d) => d.product_id)).toEqual(["2", "3"]);
	});

	it("is empty with no dislikes", () => {
		expect(groupByMember([])).toEqual([]);
	});
});
