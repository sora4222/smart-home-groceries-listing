import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { PendingRequestCard } from "#/components/pending/pending-request-card";
import type { VoiceRequest } from "#/lib/api";

const request: VoiceRequest = {
	id: "11111111-1111-1111-1111-111111111111",
	source: "webhook",
	raw_text: "milk",
	parsed_name: "milk",
	parsed_quantity: 2,
	status: "pending",
	created_at: new Date().toISOString(),
	triage_status: "approved",
	triage_reason: "Milk is sold at supermarkets.",
	triage_confidence: 0.95,
};

describe("PendingRequestCard", () => {
	it("calls onAccept with the (possibly edited) name and quantity", async () => {
		const user = userEvent.setup();
		const onAccept = vi.fn();
		render(
			<PendingRequestCard
				request={request}
				dulled={false}
				onAccept={onAccept}
				onReject={vi.fn()}
			/>,
		);

		await user.clear(screen.getByLabelText("Item name"));
		await user.type(screen.getByLabelText("Item name"), "oat milk");
		await user.click(screen.getByRole("button", { name: /accept oat milk/i }));

		expect(onAccept).toHaveBeenCalledWith(request.id, "oat milk", 2);
	});

	it("hides the reject button once dulled, so a rejected item can only be undone (accepted)", () => {
		render(
			<PendingRequestCard
				request={request}
				dulled={true}
				onAccept={vi.fn()}
				onReject={vi.fn()}
			/>,
		);

		expect(
			screen.queryByRole("button", { name: /reject/i }),
		).not.toBeInTheDocument();
		expect(screen.getByRole("button", { name: /accept/i })).toBeInTheDocument();
		expect(screen.getByText("Rejected")).toBeInTheDocument();
	});

	it("reports an edit to the name or quantity", async () => {
		const user = userEvent.setup();
		const onEdit = vi.fn();
		render(
			<PendingRequestCard
				request={request}
				dulled={false}
				onAccept={vi.fn()}
				onReject={vi.fn()}
				onEdit={onEdit}
			/>,
		);

		await user.type(screen.getByLabelText("Item name"), "s");
		await user.type(screen.getByLabelText("Qty"), "3");

		expect(onEdit).toHaveBeenCalledTimes(2);
		expect(onEdit).toHaveBeenCalledWith(request.id);
	});

	it("keeps the edits and turns its buttons off while busy", async () => {
		const user = userEvent.setup();
		const props = {
			request,
			dulled: false,
			onAccept: vi.fn(),
			onReject: vi.fn(),
		};
		const { rerender } = render(<PendingRequestCard {...props} />);
		await user.clear(screen.getByLabelText("Item name"));
		await user.type(screen.getByLabelText("Item name"), "oat milk");

		rerender(<PendingRequestCard {...props} busy />);
		expect(screen.getByRole("button", { name: /accept/i })).toBeDisabled();
		expect(screen.getByRole("button", { name: /reject/i })).toBeDisabled();

		rerender(<PendingRequestCard {...props} />);
		expect(screen.getByLabelText("Item name")).toHaveValue("oat milk");
	});
});
