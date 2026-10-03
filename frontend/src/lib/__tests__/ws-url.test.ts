import { describe, expect, it } from "vitest";

import { socketUrl } from "#/lib/ws-url";

describe("socketUrl", () => {
	it("leaves the address alone with no token", () => {
		expect(socketUrl("ws://localhost:8000/ws", null)).toBe(
			"ws://localhost:8000/ws",
		);
		expect(socketUrl("ws://localhost:8000/ws", "")).toBe(
			"ws://localhost:8000/ws",
		);
	});

	it("adds the token as ?token=", () => {
		expect(socketUrl("ws://localhost:8000/ws", "abc.def")).toBe(
			"ws://localhost:8000/ws?token=abc.def",
		);
	});

	it("adds it with & when the address already has a query", () => {
		expect(socketUrl("ws://host/ws?a=1", "abc")).toBe(
			"ws://host/ws?a=1&token=abc",
		);
	});

	it("escapes characters that would break the query", () => {
		expect(socketUrl("ws://host/ws", "a+b/c=&d")).toBe(
			"ws://host/ws?token=a%2Bb%2Fc%3D%26d",
		);
	});
});
