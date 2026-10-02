import { describe, expect, it } from "vitest";

import type { TrolleyHandoff, TrolleyHandoffLine } from "#/lib/api";
import { handoffNotice } from "#/lib/desktop/handoff-notice";

function line(outcome: TrolleyHandoffLine["outcome"]): TrolleyHandoffLine {
	return {
		grocery_item_id: crypto.randomUUID(),
		product_id: "1",
		name: "Milk",
		quantity: 1,
		outcome,
		problem: null,
	};
}

function handoff(
	status: TrolleyHandoff["status"],
	lines: TrolleyHandoffLine[],
): TrolleyHandoff {
	return { status, lines, store_name: "Woolworths" } as TrolleyHandoff;
}

describe("handoffNotice", () => {
	it("says nothing until the trolley is filled", () => {
		expect(handoffNotice(handoff("waiting_for_store_tab", []))).toBeNull();
		expect(handoffNotice(handoff("claimed_by_store_tab", []))).toBeNull();
		expect(handoffNotice(handoff("replaced", []))).toBeNull();
	});

	it("counts what went in", () => {
		expect(
			handoffNotice(handoff("filled", [line("added"), line("added")])),
		).toEqual({
			title: "Woolworths trolley filled",
			body: "2 added. Check out when ready.",
		});
	});

	it("says how many did not go in", () => {
		expect(
			handoffNotice(
				handoff("filled_with_problems", [line("added"), line("failed")]),
			)?.body,
		).toBe("1 added. 1 not added. Check out when ready.");
	});
});
