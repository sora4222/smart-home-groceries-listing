import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ItemRuleForm } from "#/components/item-rules/item-rule-form";
import type { ItemRuleDraft } from "#/lib/api";

function renderForm(initial?: ItemRuleDraft, withCancel = false) {
	const onSubmit = vi.fn().mockResolvedValue(undefined);
	const onCancel = vi.fn();
	render(
		<ItemRuleForm
			idPrefix="test"
			initial={initial}
			submitLabel={initial ? "Save rule" : "Add rule"}
			onSubmit={onSubmit}
			onCancel={withCancel ? onCancel : undefined}
		/>,
	);
	return { onSubmit, onCancel };
}

async function addName(name: string) {
	await userEvent.type(
		screen.getByLabelText("Add an item name"),
		`${name}{Enter}`,
	);
}

async function addFilter(term: string) {
	await userEvent.type(
		screen.getByLabelText("Add a product filter"),
		`${term}{Enter}`,
	);
}

describe("ItemRuleForm", () => {
	it("creates a rule from names, filters and the manual toggle", async () => {
		const { onSubmit } = renderForm();

		await addName("toilet paper");
		await addName("loo roll");
		await addFilter("3 ply");
		await userEvent.click(
			screen.getByRole("switch", { name: "Apply to manual additions" }),
		);
		await userEvent.click(screen.getByRole("button", { name: "Add rule" }));

		expect(onSubmit).toHaveBeenCalledWith({
			triggers: ["toilet paper", "loo roll"],
			filter_terms: ["3 ply"],
			apply_to_manual: true,
		});
	});

	it("leaves manual additions off unless switched on", async () => {
		const { onSubmit } = renderForm();

		await addName("milk");
		await addFilter("a2");
		await userEvent.click(screen.getByRole("button", { name: "Add rule" }));

		expect(onSubmit).toHaveBeenCalledWith(
			expect.objectContaining({ apply_to_manual: false }),
		);
	});

	it("needs at least one name and one filter", async () => {
		renderForm();
		const submit = screen.getByRole("button", { name: "Add rule" });

		expect(submit).toBeDisabled();
		await addName("milk");
		expect(submit).toBeDisabled();
		await addFilter("a2");
		expect(submit).toBeEnabled();
	});

	it("clears itself after adding a rule", async () => {
		renderForm();

		await addName("milk");
		await addFilter("a2");
		await userEvent.click(screen.getByRole("button", { name: "Add rule" }));

		expect(screen.queryByLabelText("Remove trigger milk")).toBeNull();
		expect(screen.queryByLabelText("Remove filter a2")).toBeNull();
	});

	it("starts an edit from the rule and saves the change", async () => {
		const { onSubmit } = renderForm(
			{ triggers: ["milk"], filter_terms: ["a2"], apply_to_manual: true },
			true,
		);

		await userEvent.click(screen.getByLabelText("Remove filter a2"));
		await addFilter("full cream");
		await userEvent.click(screen.getByRole("button", { name: "Save rule" }));

		expect(onSubmit).toHaveBeenCalledWith({
			triggers: ["milk"],
			filter_terms: ["full cream"],
			apply_to_manual: true,
		});
	});

	it("cancels an edit without saving", async () => {
		const { onSubmit, onCancel } = renderForm(
			{ triggers: ["milk"], filter_terms: ["a2"], apply_to_manual: false },
			true,
		);

		await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

		expect(onCancel).toHaveBeenCalledOnce();
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it("keeps the draft when saving fails", async () => {
		const onSubmit = vi.fn().mockRejectedValue(new Error("offline"));
		render(
			<ItemRuleForm idPrefix="t" submitLabel="Add rule" onSubmit={onSubmit} />,
		);

		await addName("milk");
		await addFilter("a2");
		await userEvent.click(screen.getByRole("button", { name: "Add rule" }));

		expect(screen.getByLabelText("Remove trigger milk")).toBeInTheDocument();
	});
});
