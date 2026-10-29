/** The household grocery list: `/api/grocery-items`. */
import { ApiError, request } from "#/lib/api/client";

export type GroceryItemStatus = "pending" | "active" | "committed" | "ordered";
export type GroceryItemSource = "voice" | "manual";

export interface GroceryItem {
	id: string;
	name: string;
	quantity: number;
	status: GroceryItemStatus;
	source: GroceryItemSource;
	/** Free-text annotation, `null` when the item has none. */
	note: string | null;
	/** Filter chips narrowing the later product search. Never `null`. */
	filter_terms: string[];
	added_by_user_id: string | null;
	created_at: string;
}

/** A new item typed into the add-item form. */
export interface NewGroceryItem {
	name: string;
	quantity?: number;
	note?: string;
	filter_terms?: string[];
}

/**
 * Fields an edit may change. Anything left out is kept as it was; a `note` of
 * `""` clears the annotation and a `filter_terms` array replaces the chips,
 * which is how removing one chip is expressed.
 */
export interface GroceryItemEdit {
	name?: string;
	quantity?: number;
	note?: string;
	filter_terms?: string[];
}

/**
 * What to do when an added name is already on the list. `ask` — the default —
 * makes the backend answer 409 with the clashing item so the user can choose.
 */
export type OnDuplicate = "ask" | "merge" | "separate";

export interface DuplicateItemDetail {
	existing_item: { id: string; name: string; quantity: number };
	message: string;
}

/** The query limiting a commit or release to some items. */
function onlyIds(ids?: string[]): string {
	return ids ? `?ids=${ids.map(encodeURIComponent).join(",")}` : "";
}

export const groceryApi = {
	/** Everything on the list: items under review and items committed. */
	list: () => request<GroceryItem[]>("/api/grocery-items"),
	add: (item: NewGroceryItem, onDuplicate: OnDuplicate = "ask") =>
		request<GroceryItem>(`/api/grocery-items?on_duplicate=${onDuplicate}`, {
			method: "POST",
			body: JSON.stringify(item),
		}),
	/** Puts a removed item back as it was: no duplicate check, no item rules. */
	restore: (item: NewGroceryItem) =>
		request<GroceryItem>("/api/grocery-items/restore", {
			method: "POST",
			body: JSON.stringify(item),
		}),
	update: (id: string, edit: GroceryItemEdit) =>
		request<GroceryItem>(`/api/grocery-items/${id}`, {
			method: "PATCH",
			body: JSON.stringify(edit),
		}),
	remove: (id: string) =>
		request<void>(`/api/grocery-items/${id}`, { method: "DELETE" }),
	/** Locks the reviewed list in for purchase; only `ids` when given. */
	commit: (ids?: string[]) =>
		request<GroceryItem[]>(`/api/grocery-items/commit${onlyIds(ids)}`, {
			method: "POST",
		}),
	/** Reopens a committed list for editing; only `ids` when given. */
	release: (ids?: string[]) =>
		request<GroceryItem[]>(`/api/grocery-items/release${onlyIds(ids)}`, {
			method: "POST",
		}),
};

/** Narrows an {@link ApiError} to the duplicate-item 409 the add form handles. */
export function duplicateDetail(error: unknown): DuplicateItemDetail | null {
	if (!(error instanceof ApiError) || error.status !== 409) return null;
	const body = error.body as { detail?: DuplicateItemDetail } | undefined;
	return body?.detail?.existing_item ? body.detail : null;
}
