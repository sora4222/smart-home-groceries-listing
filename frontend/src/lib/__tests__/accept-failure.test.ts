import { describe, expect, it } from "vitest";

import { acceptFailure } from "#/lib/accept-failure";
import { ApiError } from "#/lib/api/client";

describe("acceptFailure", () => {
	it("drops a card someone already accepted", () => {
		const failure = acceptFailure(new ApiError(409, { detail: "id" }), "milk");
		expect(failure).toEqual({
			message: "milk was already accepted.",
			gone: true,
		});
	});

	it("drops a card that no longer exists", () => {
		const failure = acceptFailure(new ApiError(404, { detail: "id" }), "milk");
		expect(failure.gone).toBe(true);
		expect(failure.message).toContain("no longer in the queue");
	});

	it("asks a signed-out user to sign in, keeping the card", () => {
		const failure = acceptFailure(new ApiError(401, { detail: "x" }), "milk");
		expect(failure.gone).toBe(false);
		expect(failure.message).toContain("Sign in");
	});

	it("names the status of any other server error", () => {
		const failure = acceptFailure(new ApiError(500, { detail: "x" }), "milk");
		expect(failure.gone).toBe(false);
		expect(failure.message).toContain("500");
	});

	it("says the server could not be reached when fetch itself failed", () => {
		const failure = acceptFailure(new TypeError("Failed to fetch"), "milk");
		expect(failure.gone).toBe(false);
		expect(failure.message).toContain("Could not reach the server");
	});
});
