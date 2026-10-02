import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { logEntry } from "#/components/access-logs/__tests__/fixtures";
import { useAccessLogPages } from "#/hooks/useAccessLogPages";

const list = vi.fn();
const toastError = vi.fn();

vi.mock("#/lib/api", () => ({
	api: { accessLogs: { list: (...args: unknown[]) => list(...args) } },
}));
vi.mock("sonner", () => ({ toast: { error: (m: string) => toastError(m) } }));

const firstPage = {
	entries: [logEntry({ id: 9 }), logEntry({ id: 8 })],
	next_before: 8,
};

describe("useAccessLogPages", () => {
	beforeEach(() => {
		list.mockReset();
		toastError.mockReset();
	});

	it("appends the older page and stops when there is nothing older", async () => {
		list.mockResolvedValue({
			entries: [logEntry({ id: 3 })],
			next_before: null,
		});
		const { result } = renderHook(() => useAccessLogPages(firstPage));

		await act(async () => result.current.loadOlder());

		expect(list).toHaveBeenCalledWith({ before: 8, hideHealthChecks: true });
		expect(result.current.entries.map((e) => e.id)).toEqual([9, 8, 3]);
		expect(result.current.hasOlder).toBe(false);
	});

	it("starts again from the newest rows when health checks are shown", async () => {
		list.mockResolvedValue({
			entries: [logEntry({ id: 10, path: "/api/health" })],
			next_before: null,
		});
		const { result } = renderHook(() => useAccessLogPages(firstPage));

		await act(async () => result.current.changeHealthChecks(true));

		expect(list).toHaveBeenCalledWith({
			before: undefined,
			hideHealthChecks: false,
		});
		expect(result.current.showHealthChecks).toBe(true);
		expect(result.current.entries.map((e) => e.id)).toEqual([10]);
	});

	it("keeps what is shown and says so when a load fails", async () => {
		list.mockRejectedValue(new Error("offline"));
		const { result } = renderHook(() => useAccessLogPages(firstPage));

		await act(async () => result.current.changeHealthChecks(true));

		expect(toastError).toHaveBeenCalled();
		expect(result.current.showHealthChecks).toBe(false);
		expect(result.current.entries.map((e) => e.id)).toEqual([9, 8]);
		expect(result.current.loading).toBe(false);
	});
});
