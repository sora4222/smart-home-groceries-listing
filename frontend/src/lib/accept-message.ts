/** The toast text after an intake request is accepted onto the list. */
import type { VoiceAcceptResult } from "#/lib/api/voice";

/** "milk added" for a new entry, "milk: now 3" when it added to one already there. */
export function acceptMessage(name: string, result: VoiceAcceptResult): string {
	return result.merged
		? `${result.grocery_item.name}: now ${result.grocery_item.quantity}`
		: `${name} added`;
}
