import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ruleFixture } from "#/components/item-rules/__tests__/fixtures";
import { ItemRuleCard } from "#/components/item-rules/item-rule-card";
import type { ItemRule } from "#/lib/api";

function renderCard(rule: ItemRule = ruleFixture()) {
	const onSave = vi.fn().mockResolvedValue(undefined);
	const onRemove = vi.fn().mockResolvedValue(undefined);
	render(<ItemRuleCard rule={rule} onSave={onSave} onRemove={onRemove} />);
	return { onSave, onRemove };
}

describe("ItemRuleCard", () => {
	it("shows the names a rule matches and the filters it adds", () => {
		renderCard(
			ruleFixture({
				triggers: ["toilet paper", "loo roll"],
				filter_terms: ["3 ply", "recycled"],
			}),
		);

		expect(screen.getByLabelText("Item names")).toHaveTextContent(
			/toilet paper.*loo roll/,
		);
		expect(screen.getByLabelText("Product filters")).toHaveTextContent(
			/3 ply.*recycled/,
		);
	});

	it("says whether manual additions get the rule", () => {
		renderCard(ruleFixture({ apply_to_manual: true }));

		expect(screen.getByText("Voice and manual additions")).toBeInTheDocument();
	});

	it("says a default rule is for voice only", () => {
		renderCard();

		expect(screen.getByText("Voice additions only")).toBeInTheDocument();
	});

	it("edits in place and saves the whole rule", async () => {
		const { onSave } = renderCard();

		await userEvent.click(
			screen.getByRole("button", { name: "Edit rule toilet paper" }),
		);
		await userEvent.click(
			screen.getByRole("switch", { name: "Apply to manual additions" }),
		);
		await userEvent.click(screen.getByRole("button", { name: "Save rule" }));

		expect(onSave).toHaveBeenCalledWith(ruleFixture().id, {
			triggers: ["toilet paper"],
			filter_terms: ["3 ply"],
			apply_to_manual: true,
		});
		expect(
			screen.getByRole("button", { name: "Edit rule toilet paper" }),
		).toBeInTheDocument();
	});

	it("asks before deleting a rule", async () => {
		const { onRemove } = renderCard();

		await userEvent.click(
			screen.getByRole("button", { name: "Delete rule toilet paper" }),
		);
		expect(onRemove).not.toHaveBeenCalled();

		await userEvent.click(
			screen.getByRole("button", {
				name: "Confirm deleting rule toilet paper",
			}),
		);
		expect(onRemove).toHaveBeenCalledWith(ruleFixture().id);
	});
});
