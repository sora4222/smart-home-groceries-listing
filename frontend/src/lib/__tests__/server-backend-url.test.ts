import { describe, expect, it } from "vitest";

import { resolveServerApiUrl } from "#/lib/backend-url";

describe("server API address", () => {
	it("preserves the browser API address when no internal address is set", () => {
		expect(resolveServerApiUrl("https://api.example.test", "  ")).toBe(
			"https://api.example.test",
		);
	});

	it("uses the internal container address only for server requests", () => {
		expect(
			resolveServerApiUrl("http://localhost:43001", " http://backend:8000/ "),
		).toBe("http://backend:8000");
	});

	it("rejects an internal address without an HTTP scheme", () => {
		expect(() =>
			resolveServerApiUrl("http://localhost:8000", "backend:8000"),
		).toThrow(/SERVER_API_BASE_URL/);
	});
});
