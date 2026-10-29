import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { runUndo } from "#/lib/undo-toast";

vi.mock("sonner", () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

describe("runUndo", () => {
	beforeEach(() => vi.clearAllMocks());

	it("undoes, says so, then runs after", async () => {
		const order: string[] = [];
		await runUndo({
			undo: async () => order.push("undo"),
			failure: "Could not undo",
			after: async () => order.push("after"),
		});

		expect(order).toEqual(["undo", "after"]);
		expect(toast.success).toHaveBeenCalledWith("Undone");
	});

	it("says when the undo failed, and still runs after", async () => {
		const after = vi.fn().mockResolvedValue(undefined);
		await runUndo({
			undo: () => Promise.reject(new Error("offline")),
			failure: "Could not undo",
			after,
		});

		expect(toast.error).toHaveBeenCalledWith("Could not undo");
		expect(after).toHaveBeenCalled();
	});
});
