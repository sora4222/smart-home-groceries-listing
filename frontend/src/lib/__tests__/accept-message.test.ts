import { describe, expect, it } from "vitest";

import { acceptMessage } from "#/lib/accept-message";
import type { VoiceAcceptResult } from "#/lib/api";

function result(merged: boolean, quantity: number): VoiceAcceptResult {
	return {
		merged,
		voice_request: {} as VoiceAcceptResult["voice_request"],
		grocery_item: {
			id: "i",
			name: "Milk",
			quantity,
			status: "active",
			source: "voice",
			note: null,
			filter_terms: [],
			added_by_user_id: null,
			created_at: "",
		},
	};
}

describe("acceptMessage", () => {
	it("names a new entry", () => {
		expect(acceptMessage("milk", result(false, 2))).toBe("milk added");
	});

	it("gives the new total when it added to an item already there", () => {
		expect(acceptMessage("milk", result(true, 3))).toBe("Milk: now 3");
	});
});
