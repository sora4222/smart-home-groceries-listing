import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useCountRiseToast } from "#/hooks/useCountRiseToast";
import type { ServerEvent } from "#/lib/ws";

const listeners = new Set<(event: ServerEvent) => void>();

vi.mock("#/lib/ws", () => ({
	connect: vi.fn(),
	subscribe: (listener: (event: ServerEvent) => void) => {
		listeners.add(listener);
		return () => listeners.delete(listener);
	},
}));

function push(event: ServerEvent) {
	for (const listener of listeners) listener(event);
}

describe("useCountRiseToast", () => {
	beforeEach(() => listeners.clear());

	it("does not fire for the first count, only for a rise after it", () => {
		const onRise = vi.fn();
		renderHook(() => useCountRiseToast("triage_held", onRise));

		push({ type: "triage_held", count: 2 });
		expect(onRise).not.toHaveBeenCalled();

		push({ type: "triage_held", count: 3 });
		expect(onRise).toHaveBeenCalledTimes(1);
	});

	it("ignores a fall, an unchanged count and other event types", () => {
		const onRise = vi.fn();
		renderHook(() => useCountRiseToast("triage_held", onRise));

		push({ type: "triage_held", count: 2 });
		push({ type: "triage_held", count: 1 });
		push({ type: "triage_held", count: 1 });
		push({ type: "voice_request_added", count: 9 });

		expect(onRise).not.toHaveBeenCalled();
	});

	it("stops listening when unmounted", () => {
		const { unmount } = renderHook(() =>
			useCountRiseToast("triage_held", vi.fn()),
		);
		unmount();
		expect(listeners.size).toBe(0);
	});
});
