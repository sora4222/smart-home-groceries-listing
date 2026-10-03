import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { TriageRequestCard } from "#/components/triage/triage-request-card";
import { heldRequest } from "./fixtures";

describe("TriageRequestCard", () => {
	it("shows the item, what was heard, the channel, the reason and how sure", () => {
		render(
			<TriageRequestCard
				request={heldRequest()}
				busy={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(screen.getByText("flibber")).toBeInTheDocument();
		expect(
			screen.getByText("Heard: “add flibber to the list”"),
		).toBeInTheDocument();
		expect(screen.getByText("Alexa")).toBeInTheDocument();
		expect(screen.getByText("Held for review")).toBeInTheDocument();
		expect(screen.getByText(/Not sure what this is\./)).toBeInTheDocument();
		expect(screen.getByText("(40% sure)")).toBeInTheDocument();
	});

	it("leaves out what was heard when it is just the item name, and the confidence when there is none", () => {
		render(
			<TriageRequestCard
				request={heldRequest({
					raw_text: "flibber",
					triage_confidence: null,
					triage_reason: "The checker could not be reached (timed out)",
				})}
				busy={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(screen.queryByText(/Heard:/)).not.toBeInTheDocument();
		expect(screen.queryByText(/% sure/)).not.toBeInTheDocument();
		expect(screen.getByText(/timed out/)).toBeInTheDocument();
	});

	it("labels a rejected request as rejected", () => {
		render(
			<TriageRequestCard
				request={heldRequest({ triage_status: "rejected" })}
				busy={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(screen.getByText("Rejected")).toBeInTheDocument();
	});

	it("passes the request to Accept and Reject", async () => {
		const user = userEvent.setup();
		const onAccept = vi.fn();
		const onReject = vi.fn();
		const request = heldRequest();
		render(
			<TriageRequestCard
				request={request}
				busy={false}
				onAccept={onAccept}
				onReject={onReject}
				onRestore={vi.fn()}
			/>,
		);

		await user.click(screen.getByRole("button", { name: "Accept flibber" }));
		await user.click(screen.getByRole("button", { name: "Reject flibber" }));

		expect(onAccept).toHaveBeenCalledWith(request);
		expect(onReject).toHaveBeenCalledWith(request);
	});

	it("disables both buttons while a decision is being saved", () => {
		render(
			<TriageRequestCard
				request={heldRequest()}
				busy={true}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Accept flibber" }),
		).toBeDisabled();
		expect(
			screen.getByRole("button", { name: "Reject flibber" }),
		).toBeDisabled();
	});

	it("offers to put a Google Tasks item back, and passes the request", async () => {
		const user = userEvent.setup();
		const onRestore = vi.fn();
		const request = heldRequest({ source: "tasks" });
		render(
			<TriageRequestCard
				request={request}
				busy={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={onRestore}
			/>,
		);

		expect(screen.getByText("Google Tasks")).toBeInTheDocument();
		await user.click(
			screen.getByRole("button", { name: "Put flibber back in Google Tasks" }),
		);

		expect(onRestore).toHaveBeenCalledWith(request);
	});

	it("has nothing to put back for a channel with no list", () => {
		render(
			<TriageRequestCard
				request={heldRequest({ source: "alexa" })}
				busy={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(
			screen.queryByRole("button", { name: /back in Google Tasks/ }),
		).toBeNull();
	});

	it("disables putting back while a decision is being saved", () => {
		render(
			<TriageRequestCard
				request={heldRequest({ source: "tasks" })}
				busy={true}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onRestore={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Put flibber back in Google Tasks" }),
		).toBeDisabled();
	});
});
