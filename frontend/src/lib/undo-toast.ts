/**
 * The success toast with an Undo button that every saving action shows.
 *
 * Undo runs the reversing call, says "Undone" or `failure`, and then runs
 * `after` (usually the page's refresh) either way.
 */
import type { ReactNode } from "react";
import { toast } from "sonner";

export interface UndoToast {
	/** Reverses the change. */
	undo: () => Promise<unknown>;
	/** Shown when the undo itself fails; a function picks it from the error. */
	failure: string | ((error: unknown) => string);
	/** Runs after the undo, whether it worked or not. */
	after?: () => Promise<unknown>;
	description?: ReactNode;
}

export function toastWithUndo(message: string, options: UndoToast) {
	toast.success(message, {
		description: options.description,
		action: { label: "Undo", onClick: () => runUndo(options) },
	});
}

/** Runs one undo: says how it went, then runs `after` either way. */
export async function runUndo({ undo, failure, after }: UndoToast) {
	try {
		await undo();
		toast.success("Undone");
	} catch (error) {
		toast.error(typeof failure === "string" ? failure : failure(error));
	}
	await after?.();
}
