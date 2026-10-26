import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { useElementWidth } from "#/hooks/useElementWidth";

type Callback = (entries: Array<{ contentRect: { width: number } }>) => void;

describe("useElementWidth", () => {
	const original = globalThis.ResizeObserver;
	afterEach(() => {
		globalThis.ResizeObserver = original;
	});

	it("starts at the fallback, then follows the element's width", () => {
		let report: Callback = () => {};
		globalThis.ResizeObserver = vi.fn(function (this: unknown, cb: Callback) {
			report = cb;
			return { observe: vi.fn(), disconnect: vi.fn(), unobserve: vi.fn() };
		}) as unknown as typeof ResizeObserver;

		const { result } = renderHook(() => useElementWidth<HTMLDivElement>(600));
		expect(result.current[1]).toBe(600);

		act(() => result.current[0](document.createElement("div")));
		act(() => report([{ contentRect: { width: 358.4 } }]));
		expect(result.current[1]).toBe(358);
	});
});
