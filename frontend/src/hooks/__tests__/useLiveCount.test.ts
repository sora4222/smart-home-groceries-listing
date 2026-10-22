import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useLiveCount } from "#/hooks/useLiveCount";
import type { ServerEvent } from "#/lib/ws";

const listeners = new Set<(event: ServerEvent) => void>();

vi.mock("#/lib/ws", async (original) => ({
	countIn: (await original<typeof import("#/lib/ws")>()).countIn,
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

	it("uses connection snapshots and never overwrites them with a late HTTP load", async () => {
		let resolveLoad!: (count: number) => void;
		const load = () =>
			new Promise<number>((resolve) => {
				resolveLoad = resolve;
			});
		const { result } = renderHook(() => useLiveCount("triage_held", load));
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 4, triage_held: 2 },
		});
		expect(result.current).toBe(2);
		await act(async () => resolveLoad(8));
		expect(result.current).toBe(2);
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 0, triage_held: 1 },
		});
		expect(result.current).toBe(1);
	});
});
