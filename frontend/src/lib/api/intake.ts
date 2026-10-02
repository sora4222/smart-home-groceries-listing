/**
 * Intake channel settings: `/api/intake/settings` and the Google Tasks
 * channel under `/api/intake/google-tasks`. No call here ever carries a
 * token: the backend keeps the Google sign-in to itself.
 */
import { request } from "#/lib/api/client";

/** One Google Tasks list. */
export interface TaskList {
	id: string;
	title: string;
}

/** The Google Tasks channel, as the settings page shows it. */
export interface GoogleTasksStatus {
	/** The Google OAuth client id and secret are set on the server. */
	configured: boolean;
	/** The server can store a sign-in (its encryption key is set). */
	encryption_ready: boolean;
	connected: boolean;
	connected_at: string | null;
	task_list: TaskList | null;
	enabled: boolean;
	poll_seconds: number;
	/** Polling will actually run: on, connected and a list chosen. */
	polling: boolean;
	last_polled_at: string | null;
	last_poll_error: string | null;
}

/** Everything `/settings/intake` shows. */
export interface IntakeSettings {
	google_tasks: GoogleTasksStatus;
	alexa: { configured: boolean };
	webhook: { configured: boolean };
	triage: { provider: string; model: string };
}

/** The household's Google Tasks choices. */
export interface GoogleTasksChoices {
	task_list_id: string | null;
	enabled: boolean;
	poll_seconds: number;
}

/** What one Google Tasks check did. */
export interface PollReport {
	recorded: number;
	already_seen: number;
	blank: number;
	not_deleted: number;
}

export const intakeApi = {
	settings: () => request<IntakeSettings>("/api/intake/settings"),
	startSignIn: () =>
		request<{ authorize_url: string }>("/api/intake/google-tasks/sign-in", {
			method: "POST",
		}),
	finishSignIn: (code: string, state: string) =>
		request<GoogleTasksStatus>("/api/intake/google-tasks/sign-in/finish", {
			method: "POST",
			body: JSON.stringify({ code, state }),
		}),
	disconnect: () =>
		request<void>("/api/intake/google-tasks/sign-in", { method: "DELETE" }),
	lists: () => request<TaskList[]>("/api/intake/google-tasks/lists"),
	saveChoices: (choices: GoogleTasksChoices) =>
		request<GoogleTasksStatus>("/api/intake/google-tasks", {
			method: "PUT",
			body: JSON.stringify(choices),
		}),
	pollNow: () =>
		request<PollReport>("/api/intake/google-tasks/poll", { method: "POST" }),
};
