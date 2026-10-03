import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TriageList } from "#/components/triage/triage-list";
import { heldRequest } from "./fixtures";

describe("TriageList", () => {
	it("says so when the tab is empty", () => {
		render(
			<TriageList
				requests={[]}
				emptyText="Nothing is held for review."
				busyId={null}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(screen.getByText("Nothing is held for review.")).toBeInTheDocument();
	});

	it("marks only the card being saved as busy", () => {
		const first = heldRequest();
		const second = heldRequest({
			id: "33333333-3333-3333-3333-333333333333",
			parsed_name: "zorp",
		});
		render(
			<TriageList
				requests={[first, second]}
				emptyText=""
				busyId={first.id}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Accept flibber" }),
		).toBeDisabled();
		expect(screen.getByRole("button", { name: "Accept zorp" })).toBeEnabled();
	});
});
