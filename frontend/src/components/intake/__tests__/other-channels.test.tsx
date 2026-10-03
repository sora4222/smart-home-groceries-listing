import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { OtherChannels } from "#/components/intake/other-channels";
import type { IntakeSettings } from "#/lib/api";

function settings(overrides: Partial<IntakeSettings> = {}): IntakeSettings {
	return {
		google_tasks: {} as IntakeSettings["google_tasks"],
		alexa: { configured: true },
		webhook: { configured: false },
		triage: { provider: "ollama", model: "llama3.2" },
		...overrides,
	};
}

function row(name: string) {
	return screen.getByText(name).closest("li") as HTMLElement;
}

describe("OtherChannels", () => {
	it("says which channels are set up, and which checker is in use", () => {
		render(<OtherChannels settings={settings()} />);

		expect(within(row("Alexa")).getByText("Set up")).toBeInTheDocument();
		expect(within(row("Webhook")).getByText("Not set up")).toBeInTheDocument();
		expect(within(row("Item checker")).getByText("ollama")).toBeInTheDocument();
		expect(
			within(row("Item checker")).getByText(/llama3\.2/),
		).toBeInTheDocument();
	});

	it("shows Google Keep as paused", () => {
		render(<OtherChannels settings={settings()} />);

		expect(within(row("Google Keep")).getByText("Paused")).toBeInTheDocument();
	});
});
