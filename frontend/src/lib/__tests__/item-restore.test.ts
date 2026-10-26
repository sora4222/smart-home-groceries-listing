import { beforeEach, describe, expect, it, vi } from "vitest";

import { selectionFixture } from "#/components/selections/__tests__/fixtures";
import { api, type ItemSelection } from "#/lib/api";
import { applyUndo } from "#/lib/item-restore";

const ITEM = "33333333-3333-3333-3333-333333333333";

/** The item now buys `counts` at Woolworths, oldest first. */
function now(...counts: Array<[string, number]>): ItemSelection[] {
	const [first, ...rest] = counts.map(([product_id, quantity]) =>
		selectionFixture({ store: "woolworths", product_id, quantity }),
	);
	return first ? [{ ...first, bought_with: rest }] : [];
}

const plan = (...counts: Array<[string, number]>) => ({
	store: "woolworths" as const,
	products: counts.map(([product_id, quantity]) => ({ product_id, quantity })),
	buyAt: null,
});

beforeEach(() => {
	vi.restoreAllMocks();
	vi.spyOn(api.selections, "choose").mockResolvedValue(selectionFixture());
	vi.spyOn(api.selections, "clear").mockResolvedValue(undefined);
	vi.spyOn(api.selections, "setQuantity").mockResolvedValue(undefined);
});

describe("applyUndo", () => {
	it("puts counts back without clearing or choosing again", async () => {
		vi.spyOn(api.selections, "list").mockResolvedValue(now(["a", 2], ["b", 1]));

		await applyUndo(ITEM, plan(["a", 1], ["b", 2]));

		expect(api.selections.clear).not.toHaveBeenCalled();
		expect(api.selections.choose).not.toHaveBeenCalled();
		expect(api.selections.setQuantity).toHaveBeenCalledWith(
			ITEM,
			"woolworths",
			"a",
			1,
		);
	});

	it("chooses a cleared product again, keeping the others", async () => {
		vi.spyOn(api.selections, "list").mockResolvedValue(now(["a", 3]));

		await applyUndo(ITEM, plan(["a", 2], ["b", 1]));

		expect(api.selections.choose).toHaveBeenCalledWith(ITEM, {
			store: "woolworths",
			product_id: "b",
		});
		expect(api.selections.clear).not.toHaveBeenCalled();
	});

	it("clears only the product the change added", async () => {
		vi.spyOn(api.selections, "list").mockResolvedValue(now(["a", 2], ["b", 1]));

		await applyUndo(ITEM, plan(["a", 3]));

		expect(api.selections.clear).toHaveBeenCalledWith(ITEM, "woolworths", "b");
		expect(api.selections.choose).not.toHaveBeenCalled();
	});

	it("leaves the current products when the store refuses the old one", async () => {
		vi.spyOn(api.selections, "list").mockResolvedValue(now(["b", 1]));
		vi.mocked(api.selections.choose).mockRejectedValue(new Error("503"));

		await expect(applyUndo(ITEM, plan(["a", 1]))).rejects.toThrow();

		expect(api.selections.clear).not.toHaveBeenCalled();
	});
});
