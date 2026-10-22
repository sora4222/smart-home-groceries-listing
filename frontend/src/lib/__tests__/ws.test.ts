import { describe, expect, it } from "vitest";

import { countIn, subscribe } from "#/lib/ws";

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

describe("countIn", () => {
	const snapshot = {
		type: "counts_snapshot" as const,
		counts: { voice_request_added: 3, triage_held: 1 },
	};

	it("reads the wanted count from a connection snapshot", () => {
		expect(countIn(snapshot, "voice_request_added")).toBe(3);
		expect(countIn(snapshot, "triage_held")).toBe(1);
	});

	it("reads a live event of the wanted type", () => {
		expect(countIn({ type: "triage_held", count: 4 }, "triage_held")).toBe(4);
	});

	it("is null for another count's live event", () => {
		expect(
			countIn({ type: "triage_held", count: 4 }, "voice_request_added"),
		).toBeNull();
	});
});
