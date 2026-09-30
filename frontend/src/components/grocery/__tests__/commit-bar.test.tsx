import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { CommitBar } from "#/components/grocery/commit-bar";

describe("CommitBar", () => {
	it("offers to commit the items under review, counted", () => {
		render(
			<CommitBar
				activeCount={3}
				committedCount={0}
				onCommit={vi.fn()}
				onRelease={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Ready to order (3 items)" }),
		).toBeInTheDocument();
	});

	it("counts a single item in the singular", () => {
		render(
			<CommitBar
				activeCount={1}
				committedCount={0}
				onCommit={vi.fn()}
				onRelease={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Ready to order (1 item)" }),
		).toBeInTheDocument();
	});

	it("stays out of the way when there is nothing to commit", () => {
		render(
			<CommitBar
				activeCount={0}
				committedCount={0}
				onCommit={vi.fn()}
				onRelease={vi.fn()}
			/>,
		);

		expect(screen.queryByRole("button")).not.toBeInTheDocument();
	});

	it("commits when asked", async () => {
		const onCommit = vi.fn().mockResolvedValue(undefined);
		render(
			<CommitBar
				activeCount={2}
				committedCount={0}
				onCommit={onCommit}
				onRelease={vi.fn()}
			/>,
		);

		await userEvent.click(
			screen.getByRole("button", { name: "Ready to order (2 items)" }),
		);

		expect(onCommit).toHaveBeenCalledOnce();
	});

	it("says the list is locked and offers to release it", async () => {
		const onRelease = vi.fn().mockResolvedValue(undefined);
		render(
			<CommitBar
				activeCount={0}
				committedCount={4}
				onCommit={vi.fn()}
				onRelease={onRelease}
			/>,
		);

		expect(
			screen.getByText("4 items locked in for purchase."),
		).toBeInTheDocument();
		await userEvent.click(
			screen.getByRole("button", { name: "Release for editing" }),
		);

		expect(onRelease).toHaveBeenCalledOnce();
	});

	it("shows both states when an item is added after a commit", () => {
		render(
			<CommitBar
				activeCount={2}
				committedCount={1}
				onCommit={vi.fn()}
				onRelease={vi.fn()}
			/>,
		);

		expect(
			screen.getByText("1 item locked in for purchase."),
		).toBeInTheDocument();
		expect(
			screen.getByRole("button", { name: "Ready to order (2 items)" }),
		).toBeInTheDocument();
	});
});
