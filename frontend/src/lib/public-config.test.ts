import { describe, expect, it } from "vitest";
import { publicConfigScript, selectPublicConfig } from "./public-config";

describe("public startup settings", () => {
	it("exposes only the explicit public allowlist", () => {
		expect(
			selectPublicConfig({
				VITE_API_BASE_URL: "",
				CLERK_SECRET_KEY: "private",
				OP_SERVICE_ACCOUNT_TOKEN: "private",
				VITE_UNREVIEWED_SECRET: "private",
			}),
		).toEqual({ VITE_API_BASE_URL: "" });
	});
	it("cannot break out of its script element", () => {
		const script = publicConfigScript({
			VITE_API_BASE_URL: "</script><script>alert(1)</script>",
		});
		expect(script).not.toContain("<");
		expect(script).toContain("\\u003c");
	});
});
