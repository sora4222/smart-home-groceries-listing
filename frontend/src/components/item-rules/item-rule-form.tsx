import { useState } from "react";

import { TermListField } from "#/components/terms/term-list-field";
import { Button } from "#/components/ui/button";
import { Switch } from "#/components/ui/switch";
import {
	type ItemRuleDraft,
	MAX_FILTER_TERM_LENGTH,
	MAX_TRIGGER_LENGTH,
} from "#/lib/api";

interface ItemRuleFormProps {
	/** Unique on the page; prefixes every field's id. */
	idPrefix: string;
	/** The rule being edited. Omitted when adding a new rule. */
	initial?: ItemRuleDraft;
	submitLabel: string;
	/** Resolves once saved, or rejects to keep the draft for a retry. */
	onSubmit: (draft: ItemRuleDraft) => Promise<void>;
	/** Shown as a Cancel button when given — editing, not adding. */
	onCancel?: () => void;
}

const EMPTY: ItemRuleDraft = {
	triggers: [],
	filter_terms: [],
	apply_to_manual: false,
};

/**
 * The item rule form: the item names a rule matches, the filters it adds, and
 * the spec's per-rule "Apply to manual additions" toggle, off by default.
 *
 * Used both to add a rule (no `initial`, clears itself after saving) and to
 * edit one in place (`initial` and `onCancel`). Every draft is local state, so
 * cancelling leaves the rule exactly as it was.
 */
export function ItemRuleForm({
	idPrefix,
	initial,
	submitLabel,
	onSubmit,
	onCancel,
}: ItemRuleFormProps) {
	const start = initial ?? EMPTY;
	const [triggers, setTriggers] = useState(start.triggers);
	const [terms, setTerms] = useState(start.filter_terms);
	const [applyToManual, setApplyToManual] = useState(start.apply_to_manual);
	const [busy, setBusy] = useState(false);

	const complete = triggers.length > 0 && terms.length > 0;

	async function submit() {
		if (!complete || busy) return;
		setBusy(true);
		try {
			await onSubmit({
				triggers,
				filter_terms: terms,
				apply_to_manual: applyToManual,
			});
			if (!initial) {
				setTriggers([]);
				setTerms([]);
				setApplyToManual(false);
			}
		} catch {
			// The page reports the failure; the draft stays for a retry.
		} finally {
			setBusy(false);
		}
	}

	return (
		<div className="flex flex-col gap-4 p-4">
			<TermListField
				id={`${idPrefix}-trigger`}
				label="Item names"
				noun="trigger"
				inputLabel="Add an item name"
				addLabel="Add name"
				placeholder="toilet paper"
				maxLength={MAX_TRIGGER_LENGTH}
				terms={triggers}
				onChange={setTriggers}
			/>

			<TermListField
				id={`${idPrefix}-filter`}
				label="Product filters"
				noun="filter"
				inputLabel="Add a product filter"
				addLabel="Add filter"
				placeholder="3 ply"
				maxLength={MAX_FILTER_TERM_LENGTH}
				terms={terms}
				onChange={setTerms}
			/>

			<div className="flex items-center gap-3">
				<Switch
					id={`${idPrefix}-manual`}
					checked={applyToManual}
					onCheckedChange={setApplyToManual}
				/>
				<label htmlFor={`${idPrefix}-manual`} className="text-sm">
					Apply to manual additions
				</label>
			</div>
			<p className="-mt-2 text-xs text-muted-foreground">
				Voice items always get a rule's filters. Items typed into the list only
				do when this is on.
			</p>

			<div className="flex flex-wrap items-center gap-2">
				<Button
					type="button"
					size="sm"
					onClick={submit}
					disabled={busy || !complete}
				>
					{busy ? "Saving…" : submitLabel}
				</Button>
				{onCancel && (
					<Button type="button" size="sm" variant="ghost" onClick={onCancel}>
						Cancel
					</Button>
				)}
				{!complete && (
					<span className="text-xs text-muted-foreground">
						Add at least one item name and one filter.
					</span>
				)}
			</div>
		</div>
	);
}
