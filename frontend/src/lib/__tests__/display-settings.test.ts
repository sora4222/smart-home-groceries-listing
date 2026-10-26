import { afterEach, describe, expect, it, vi } from "vitest";

import {
	DEFAULT_DISPLAY,
	parseDisplay,
	readDisplayRaw,
	saveDisplay,
	subscribeDisplay,
} from "#/lib/display-settings";

afterEach(() => window.localStorage.clear());

describe("parseDisplay", () => {
	it("gives the defaults for nothing stored, or junk", () => {
		expect(parseDisplay(null)).toEqual(DEFAULT_DISPLAY);
		expect(parseDisplay("{not json")).toEqual(DEFAULT_DISPLAY);
		expect(parseDisplay('{"imageSize":"huge","showWasPrice":"no"}')).toEqual(
			DEFAULT_DISPLAY,
		);
	});

	it("reads what was saved", () => {
		expect(
			parseDisplay(
				'{"imageSize":"lg","showWasPrice":false,"showUnavailable":false}',
			),
		).toEqual({ imageSize: "lg", showWasPrice: false, showUnavailable: false });
	});
});

describe("saveDisplay", () => {
	it("stores the settings and tells subscribers", () => {
		const onChange = vi.fn();
		const stop = subscribeDisplay(onChange);

		saveDisplay({ ...DEFAULT_DISPLAY, imageSize: "sm" });

		expect(parseDisplay(readDisplayRaw()).imageSize).toBe("sm");
		expect(onChange).toHaveBeenCalledOnce();
		stop();
	});
});
