/**
 * The one place the web app talks to the desktop app (Tauri). Outside the
 * desktop app `isDesktop()` is false and nothing here calls anything.
 * Spec: `docs/features/FEATURE_DESKTOP_APP.md`.
 */
import { invoke } from "@tauri-apps/api/core";

/** True when the page runs inside the desktop app. */
export function isDesktop(): boolean {
	return (
		typeof window !== "undefined" &&
		(window as Window & { __TAURI_INTERNALS__?: unknown })
			.__TAURI_INTERNALS__ !== undefined
	);
}

/**
 * Calls a desktop app command. Rejects with a plain sentence outside the
 * desktop app; the app's own refusals arrive as sentences too.
 */
export async function invokeDesktop<T>(
	command: string,
	args?: Record<string, unknown>,
): Promise<T> {
	if (!isDesktop()) {
		throw new Error("This works only in the desktop app.");
	}
	try {
		return await invoke<T>(command, args);
	} catch (err) {
		console.error(`[desktop] ${command} failed`, err);
		throw err instanceof Error ? err : new Error(String(err));
	}
}
