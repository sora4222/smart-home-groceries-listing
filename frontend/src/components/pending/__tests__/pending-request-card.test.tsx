import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { PendingRequestCard } from "#/components/pending/pending-request-card";
import type { VoiceRequest } from "#/lib/api";

const request: VoiceRequest = {
	id: "11111111-1111-1111-1111-111111111111",
	raw_text: "milk",
	parsed_name: "milk",
	parsed_quantity: 2,
	status: "pending",
	created_at: new Date().toISOString(),
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
});
