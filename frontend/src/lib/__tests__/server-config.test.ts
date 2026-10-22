import { afterEach, expect, it, vi } from "vitest";

afterEach(() => {
	vi.unstubAllEnvs();
	vi.resetModules();
});

it.each([true, false])("selects the API for SSR=%s", async (ssr) => {
	vi.stubEnv("SSR", ssr);
	vi.stubEnv("VITE_API_BASE_URL", "http://localhost:43001");
	vi.stubEnv("VITE_WS_URL", "");
	vi.stubEnv("SERVER_API_BASE_URL", "http://backend:8000");
	const config = await import("#/lib/config");
	expect(config.API_BASE_URL).toBe(
		ssr ? "http://backend:8000" : "http://localhost:43001",
	);
	expect(config.WS_URL).toBe("ws://localhost:43001/ws");
});
