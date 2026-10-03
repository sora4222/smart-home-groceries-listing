import { describe, expect, it } from "vitest";

import { resolveAuthMode } from "#/lib/auth-mode";

describe("resolveAuthMode", () => {
	it("is on when a publishable key is set", () => {
		expect(resolveAuthMode({ VITE_CLERK_PUBLISHABLE_KEY: "pk_test_abc" })).toBe(
			"clerk",
		);
	});

	it("is off with no key, or a blank one", () => {
		expect(resolveAuthMode({})).toBe("off");
		expect(resolveAuthMode({ VITE_CLERK_PUBLISHABLE_KEY: "  " })).toBe("off");
	});

	it("can be forced off even with a key, for the e2e tests", () => {
		expect(
			resolveAuthMode({
				VITE_AUTH_MODE: "off",
				VITE_CLERK_PUBLISHABLE_KEY: "pk_test_abc",
			}),
		).toBe("off");
		expect(
			resolveAuthMode({
				VITE_AUTH_MODE: " OFF ",
				VITE_CLERK_PUBLISHABLE_KEY: "pk_test_abc",
			}),
		).toBe("off");
	});

	it("ignores any other VITE_AUTH_MODE value and follows the key", () => {
		expect(
			resolveAuthMode({
				VITE_AUTH_MODE: "clerk",
				VITE_CLERK_PUBLISHABLE_KEY: "",
			}),
		).toBe("off");
		expect(
			resolveAuthMode({
				VITE_AUTH_MODE: "yes",
				VITE_CLERK_PUBLISHABLE_KEY: "pk_test_abc",
			}),
		).toBe("clerk");
	});
});
