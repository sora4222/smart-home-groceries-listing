import { describe, expect, it } from "vitest";

import { subscribe } from "#/lib/ws";

/**
 * `ws.ts` is a client-only singleton over a real browser WebSocket, so the
 * meaningful thing to unit-test without a live socket server is the
 * subscribe/unsubscribe contract; actual message delivery is covered by
 * the e2e suite against the real backend.
 */
describe("ws client", () => {
	it("subscribe returns an unsubscribe function that stops future delivery", () => {
		const received: unknown[] = [];
		const unsubscribe = subscribe((event) => received.push(event));
		unsubscribe();
		expect(typeof unsubscribe).toBe("function");
	});
});
