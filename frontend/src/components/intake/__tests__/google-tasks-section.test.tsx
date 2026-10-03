import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GoogleTasksSection } from "#/components/intake/google-tasks-section";
import type { GoogleTasksStatus } from "#/lib/api";

const intake = vi.hoisted(() => ({
	settings: vi.fn(),
	startSignIn: vi.fn(),
	finishSignIn: vi.fn(),
	disconnect: vi.fn(),
	lists: vi.fn(),
	saveChoices: vi.fn(),
	pollNow: vi.fn(),
}));
const toast = vi.hoisted(() =>
	Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn() }),
);

vi.mock("#/lib/api", () => ({ api: { intake } }));
vi.mock("sonner", () => ({ toast }));

function status(overrides: Partial<GoogleTasksStatus> = {}): GoogleTasksStatus {
	return {
		configured: true,
		encryption_ready: true,
		connected: false,
		connected_at: null,
		task_list: null,
		enabled: false,
		poll_seconds: 60,
		polling: false,
		last_polled_at: null,
		last_poll_error: null,
		...overrides,
	};
}

const connected = status({
	connected: true,
	connected_at: "2026-10-02T03:00:00Z",
});
const watching = status({
	...connected,
	task_list: { id: "g", title: "Groceries" },
	enabled: true,
	polling: true,
});

describe("GoogleTasksSection", () => {
	beforeEach(() => {
		vi.clearAllMocks();
		intake.lists.mockResolvedValue([
			{ id: "g", title: "Groceries" },
			{ id: "m", title: "My Tasks" },
		]);
	});

	it("says which server setting is missing before anything else", () => {
		const { rerender } = render(
			<GoogleTasksSection initial={status({ configured: false })} />,
		);
		expect(screen.getByText(/GOOGLE_CLIENT_ID/)).toBeInTheDocument();
		expect(
			screen.queryByRole("button", { name: "Connect Google Tasks" }),
		).toBeNull();

		rerender(
			<GoogleTasksSection
				key="b"
				initial={status({ encryption_ready: false })}
			/>,
		);
		expect(screen.getByText(/CREDENTIAL_ENCRYPTION_KEY/)).toBeInTheDocument();
	});

	it("sends the browser to Google's sign-in page", async () => {
		const assign = vi.fn();
		vi.stubGlobal("location", { ...window.location, assign });
		intake.startSignIn.mockResolvedValue({ authorize_url: "https://google/x" });

		render(<GoogleTasksSection initial={status()} />);
		await userEvent.click(
			screen.getByRole("button", { name: "Connect Google Tasks" }),
		);

		await waitFor(() =>
			expect(assign).toHaveBeenCalledWith("https://google/x"),
		);
		vi.unstubAllGlobals();
	});

	it("shows the server's reason when the sign-in cannot start", async () => {
		intake.startSignIn.mockRejectedValue({
			body: { detail: "Server configuration error" },
		});

		render(<GoogleTasksSection initial={status()} />);
		await userEvent.click(
			screen.getByRole("button", { name: "Connect Google Tasks" }),
		);

		await waitFor(() =>
			expect(toast.error).toHaveBeenCalledWith("Server configuration error"),
		);
	});

	it("saves the chosen list, the switch and how often", async () => {
		const user = userEvent.setup();
		intake.saveChoices.mockResolvedValue(watching);

		render(<GoogleTasksSection initial={connected} />);
		expect(
			screen.getByRole("switch", { name: "Check this list" }),
		).toBeDisabled();

		await user.click(screen.getByRole("combobox", { name: "Grocery list" }));
		await user.click(await screen.findByRole("option", { name: "Groceries" }));
		await user.click(
			screen.getByRole("combobox", { name: "How often to check" }),
		);
		await user.click(screen.getByRole("option", { name: "Every 5 minutes" }));
		await user.click(screen.getByRole("switch", { name: "Check this list" }));
		await user.click(screen.getByRole("button", { name: "Save" }));

		expect(intake.saveChoices).toHaveBeenCalledWith({
			task_list_id: "g",
			enabled: true,
			poll_seconds: 300,
		});
		expect(await screen.findByText("On")).toBeInTheDocument();
	});

	it("checks now and says what it found", async () => {
		intake.pollNow.mockResolvedValue({
			recorded: 2,
			already_seen: 0,
			blank: 0,
			not_deleted: 0,
		});
		intake.settings.mockResolvedValue({ google_tasks: watching });

		render(<GoogleTasksSection initial={watching} />);
		await userEvent.click(screen.getByRole("button", { name: "Check now" }));

		await waitFor(() =>
			expect(toast.success).toHaveBeenCalledWith("2 new items found."),
		);
	});

	it("cannot check while checking is off", () => {
		render(<GoogleTasksSection initial={connected} />);

		expect(screen.getByRole("button", { name: "Check now" })).toBeDisabled();
	});

	it("shows why the last check failed", () => {
		render(
			<GoogleTasksSection
				initial={{ ...watching, last_poll_error: "Google said no" }}
			/>,
		);

		expect(screen.getByRole("alert")).toHaveTextContent("Google said no");
	});

	it("disconnects, after which only Connect is offered", async () => {
		intake.disconnect.mockResolvedValue(undefined);

		render(<GoogleTasksSection initial={watching} />);
		await userEvent.click(screen.getByRole("button", { name: "Disconnect" }));

		expect(
			await screen.findByRole("button", { name: "Connect Google Tasks" }),
		).toBeInTheDocument();
		expect(screen.getByText("Off")).toBeInTheDocument();
	});
});
