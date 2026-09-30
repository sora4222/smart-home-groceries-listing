/** Item rules: `/api/item-rules`, managed on `/settings/item-rules`. */
import { request } from "#/lib/api/client";

/**
 * A persistent filter attached to grocery item names. When an item whose name
 * matches one of `triggers` reaches the list, it gets `filter_terms` as chips —
 * always when accepted from voice, and when typed into the web app only if
 * `apply_to_manual` is on.
 */
export interface ItemRule {
	id: string;
	/** Item names or phrases to match, case-insensitively, on whole words. */
	triggers: string[];
	/** The chips a matching item receives. */
	filter_terms: string[];
	/** Whether items typed into the web app get the rule too. */
	apply_to_manual: boolean;
	created_at: string;
	updated_at: string;
}

/** What the rule form produces, for a new rule or an edit. */
export interface ItemRuleDraft {
	triggers: string[];
	filter_terms: string[];
	apply_to_manual: boolean;
}

/** Longest single trigger the backend accepts. */
export const MAX_TRIGGER_LENGTH = 100;
/** Longest single filter term the backend accepts. */
export const MAX_FILTER_TERM_LENGTH = 60;

export const itemRulesApi = {
	/** Every rule, in the order they were made. */
	list: () => request<ItemRule[]>("/api/item-rules"),
	create: (draft: ItemRuleDraft) =>
		request<ItemRule>("/api/item-rules", {
			method: "POST",
			body: JSON.stringify(draft),
		}),
	/** Fields left out are kept; a list that is sent replaces the stored one. */
	update: (id: string, edit: Partial<ItemRuleDraft>) =>
		request<ItemRule>(`/api/item-rules/${id}`, {
			method: "PATCH",
			body: JSON.stringify(edit),
		}),
	/** Chips the rule already put on items stay where they are. */
	remove: (id: string) =>
		request<void>(`/api/item-rules/${id}`, { method: "DELETE" }),
};
