import { describe, expect, it } from "vitest";

import { resolveBackendUrls } from "#/lib/backend-url";

describe("resolveBackendUrls", () => {
	it("uses the same-machine backend when nothing is set", () => {
		expect(resolveBackendUrls({})).toEqual({
			apiBaseUrl: "http://localhost:8000",
			wsUrl: "ws://localhost:8000/ws",
		});
	});

	it("treats blank values as not set", () => {
		expect(
			resolveBackendUrls({ VITE_API_BASE_URL: "  ", VITE_WS_URL: "" }),
		).toEqual({
			apiBaseUrl: "http://localhost:8000",
			wsUrl: "ws://localhost:8000/ws",
		});
	});

	it("derives a secure socket from an https backend", () => {
		expect(
			resolveBackendUrls({
				VITE_API_BASE_URL: "https://grocery-api.example.com",
			}),
		).toEqual({
			apiBaseUrl: "https://grocery-api.example.com",
			wsUrl: "wss://grocery-api.example.com/ws",
		});
	});

	it("drops trailing slashes so paths join cleanly", () => {
		expect(
			resolveBackendUrls({ VITE_API_BASE_URL: "http://192.168.1.226:8000//" })
				.apiBaseUrl,
		).toBe("http://192.168.1.226:8000");
	});

	it("keeps an explicit socket address", () => {
		expect(
			resolveBackendUrls({
				VITE_API_BASE_URL: "https://grocery-api.example.com",
				VITE_WS_URL: " wss://sockets.example.com/ws ",
			}).wsUrl,
		).toBe("wss://sockets.example.com/ws");
	});

	it("refuses a backend address without http:// or https://", () => {
		expect(() =>
			resolveBackendUrls({ VITE_API_BASE_URL: "grocery-api.example.com" }),
		).toThrow(/VITE_API_BASE_URL/);
	});

	it("refuses a socket address without ws:// or wss://", () => {
		expect(() =>
			resolveBackendUrls({ VITE_WS_URL: "https://grocery-api.example.com/ws" }),
		).toThrow(/VITE_WS_URL/);
	});
});
