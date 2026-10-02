import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useLiveCount } from "#/hooks/useLiveCount";
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
	act(() => {
		for (const listener of listeners) listener(event);
	});
}

describe("useLiveCount", () => {
	beforeEach(() => listeners.clear());

	it("starts from the loaded count, then follows pushed events of its type", async () => {
		const load = () => Promise.resolve(4);
		const { result } = renderHook(() => useLiveCount("triage_held", load));

		await waitFor(() => expect(result.current).toBe(4));
		push({ type: "voice_request_added", count: 9 });
		expect(result.current).toBe(4);
		push({ type: "triage_held", count: 1 });
		expect(result.current).toBe(1);
	});

	it("stays at 0 when the first load fails", async () => {
		const load = () => Promise.reject(new Error("offline"));
		const { result } = renderHook(() => useLiveCount("triage_held", load));

		await Promise.resolve();
		expect(result.current).toBe(0);
	});
});
