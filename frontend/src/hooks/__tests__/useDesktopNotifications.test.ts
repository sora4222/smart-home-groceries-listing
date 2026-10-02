import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { TrolleyHandoff } from "#/lib/api";
import type { ServerEvent } from "#/lib/ws";

const { listeners, notify, desktopState } = vi.hoisted(() => ({
	listeners: new Set<(event: ServerEvent) => void>(),
	notify: vi.fn(),
	desktopState: { on: true },
}));

vi.mock("#/lib/ws", () => ({
	connect: vi.fn(),
	subscribe: (listener: (event: ServerEvent) => void) => {
		listeners.add(listener);
		return () => listeners.delete(listener);
	},
}));
vi.mock("#/lib/desktop/bridge", () => ({ isDesktop: () => desktopState.on }));
vi.mock("#/lib/desktop/commands", () => ({ desktop: { notify } }));

import {
	HELD_ITEM_NOTICE,
	NEW_ITEM_NOTICE,
	useDesktopNotifications,
} from "#/hooks/useDesktopNotifications";
import { useHandoffNotice } from "#/hooks/useHandoffNotice";

function push(event: ServerEvent) {
	for (const listener of listeners) listener(event);
}

describe("desktop notifications", () => {
	beforeEach(() => {
		listeners.clear();
		notify.mockReset().mockResolvedValue(undefined);
		desktopState.on = true;
	});
	afterEach(() => vi.clearAllMocks());

	it("notifies when a new item arrives or one is held", () => {
		renderHook(() => useDesktopNotifications());
		push({ type: "voice_request_added", count: 0 });
		push({ type: "triage_held", count: 0 });
		push({ type: "voice_request_added", count: 1 });
		push({ type: "triage_held", count: 1 });

		expect(notify.mock.calls).toEqual([
			[NEW_ITEM_NOTICE.title, NEW_ITEM_NOTICE.body],
			[HELD_ITEM_NOTICE.title, HELD_ITEM_NOTICE.body],
		]);
	});

	it("stays quiet in a normal browser", () => {
		desktopState.on = false;
		renderHook(() => useDesktopNotifications());
		push({ type: "voice_request_added", count: 0 });
		push({ type: "voice_request_added", count: 1 });
		expect(notify).not.toHaveBeenCalled();
	});

	it("announces a filled trolley once", () => {
		const waiting = {
			id: "h1",
			status: "claimed_by_store_tab",
			store_name: "Woolworths",
			lines: [],
		} as unknown as TrolleyHandoff;
		const { rerender } = renderHook(
			({ handoff }) => useHandoffNotice(handoff),
			{ initialProps: { handoff: waiting } },
		);
		expect(notify).not.toHaveBeenCalled();

		const filled = { ...waiting, status: "filled" } as TrolleyHandoff;
		rerender({ handoff: filled });
		rerender({ handoff: { ...filled } });

		expect(notify).toHaveBeenCalledTimes(1);
		expect(notify).toHaveBeenCalledWith(
			"Woolworths trolley filled",
			"0 added. Check out when ready.",
		);
	});
});
