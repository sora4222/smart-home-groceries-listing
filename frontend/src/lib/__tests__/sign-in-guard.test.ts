import { describe, expect, it } from "vitest";

import {
	guardDecision,
	isAuthPath,
	needsSessionCheck,
} from "#/lib/sign-in-guard";

describe("isAuthPath", () => {
	it("matches the sign-in page and Clerk's steps under it", () => {
		expect(isAuthPath("/sign-in")).toBe(true);
		expect(isAuthPath("/sign-in/")).toBe(true);
		expect(isAuthPath("/sign-in/factor-one")).toBe(true);
		expect(isAuthPath("/sign-up")).toBe(true);
		expect(isAuthPath("/sign-up/verify-email-address")).toBe(true);
	});

	it("does not match pages that only start with the same letters", () => {
		expect(isAuthPath("/sign-inside")).toBe(false);
		expect(isAuthPath("/sign-upper")).toBe(false);
		expect(isAuthPath("/")).toBe(false);
		expect(isAuthPath("/settings/item-rules")).toBe(false);
	});
});

describe("needsSessionCheck", () => {
	it("never asks the server with sign-in off", () => {
		expect(needsSessionCheck("off", "/")).toBe(false);
	});

	it("does not ask on the sign-in or sign-up page", () => {
		expect(needsSessionCheck("clerk", "/sign-in")).toBe(false);
		expect(needsSessionCheck("clerk", "/sign-up")).toBe(false);
	});

	it("asks on every other page", () => {
		expect(needsSessionCheck("clerk", "/pending")).toBe(true);
	});
});

describe("guardDecision", () => {
	it("sends a signed-out visitor to sign in", () => {
		expect(guardDecision("clerk", "/order", false)).toBe("sign-in");
	});

	it("lets a signed-in visitor stay", () => {
		expect(guardDecision("clerk", "/order", true)).toBe("stay");
	});

	it("never loops on the sign-in page", () => {
		expect(guardDecision("clerk", "/sign-in", false)).toBe("stay");
	});

	it("lets everyone stay with sign-in off", () => {
		expect(guardDecision("off", "/order", false)).toBe("stay");
	});
});
