/**
 * What the Pending Requests screen says when accepting a request fails for a
 * reason other than the duplicate-item question (that one gets a prompt).
 *
 * `gone` tells the screen the card is stale — someone else already decided
 * it, or it no longer exists — so it should leave the queue rather than
 * invite a retry that can only fail again.
 */
import { ApiError } from "#/lib/api/client";

export interface AcceptFailure {
	message: string;
	gone: boolean;
}

/** Turns an accept error into the toast text and whether to drop the card. */
export function acceptFailure(error: unknown, name: string): AcceptFailure {
	if (error instanceof ApiError && error.status === 409) {
		return { message: `${name} was already accepted.`, gone: true };
	}
	if (error instanceof ApiError && error.status === 404) {
		return { message: `${name} is no longer in the queue.`, gone: true };
	}
	if (error instanceof ApiError && error.status === 401) {
		return {
			message: "You are signed out. Sign in again, then accept it.",
			gone: false,
		};
	}
	if (error instanceof ApiError) {
		return {
			message: `Could not accept ${name} — the server answered ${error.status}. Try again.`,
			gone: false,
		};
	}
	return {
		message: `Could not reach the server to accept ${name}. Check the connection and try again.`,
		gone: false,
	};
}
