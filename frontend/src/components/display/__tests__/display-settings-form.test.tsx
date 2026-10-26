import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { DisplaySettingsForm } from "#/components/display/display-settings-form";
import { DEFAULT_DISPLAY } from "#/lib/display-settings";

describe("DisplaySettingsForm", () => {
	it("changes the picture size", async () => {
		const onChange = vi.fn();
		render(<DisplaySettingsForm value={DEFAULT_DISPLAY} onChange={onChange} />);

		await userEvent.click(
			screen.getByRole("combobox", { name: "Picture size" }),
		);
		await userEvent.click(await screen.findByRole("option", { name: "Large" }));

		expect(onChange).toHaveBeenCalledWith({
			...DEFAULT_DISPLAY,
			imageSize: "lg",
		});
	});

	it("hides the original price when its switch is turned on", async () => {
		const onChange = vi.fn();
		render(<DisplaySettingsForm value={DEFAULT_DISPLAY} onChange={onChange} />);

		await userEvent.click(
			screen.getByRole("switch", { name: "Do not display original price" }),
		);

		expect(onChange).toHaveBeenCalledWith({
			...DEFAULT_DISPLAY,
			showWasPrice: false,
		});
	});

	it("hides unavailable items when their switch is turned off", async () => {
		const onChange = vi.fn();
		render(<DisplaySettingsForm value={DEFAULT_DISPLAY} onChange={onChange} />);

		await userEvent.click(
			screen.getByRole("switch", { name: "Display unavailable items" }),
		);

		expect(onChange).toHaveBeenCalledWith({
			...DEFAULT_DISPLAY,
			showUnavailable: false,
		});
	});
});
