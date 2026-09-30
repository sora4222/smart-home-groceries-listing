import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { DuplicatePrompt } from "#/components/grocery/duplicate-prompt";
import type { DuplicateItemDetail } from "#/lib/api";

const detail: DuplicateItemDetail = {
	existing_item: {
		id: "22222222-2222-2222-2222-222222222222",
		name: "milk",
		quantity: 2,
	},
	message:
		"milk is already on the list. Add another or update the existing quantity?",
};

describe("DuplicatePrompt", () => {
	it("asks the spec's question and shows what is already there", () => {
		render(
			<DuplicatePrompt
				detail={detail}
				onMerge={vi.fn()}
				onSeparate={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		expect(screen.getByText(detail.message)).toBeInTheDocument();
		expect(screen.getByText(/milk ×2/)).toBeInTheDocument();
	});

	it("offers both ways out and cancelling", async () => {
		const onMerge = vi.fn();
		const onSeparate = vi.fn();
		const onCancel = vi.fn();
		render(
			<DuplicatePrompt
				detail={detail}
				onMerge={onMerge}
				onSeparate={onSeparate}
				onCancel={onCancel}
			/>,
		);

		await userEvent.click(
			screen.getByRole("button", { name: "Update the quantity" }),
		);
		await userEvent.click(
			screen.getByRole("button", { name: "Add a separate entry" }),
		);
		await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

		expect(onMerge).toHaveBeenCalledOnce();
		expect(onSeparate).toHaveBeenCalledOnce();
		expect(onCancel).toHaveBeenCalledOnce();
	});
});
