import { useState } from "react";

import { ItemRuleForm } from "#/components/item-rules/item-rule-form";
import { TermChips } from "#/components/terms/term-chips";
import { Button } from "#/components/ui/button";
import { Card, CardContent } from "#/components/ui/card";
import type { ItemRule, ItemRuleDraft } from "#/lib/api";

interface ItemRuleCardProps {
	rule: ItemRule;
	/** Resolves once saved, or rejects to keep the editor open. */
	onSave: (id: string, draft: ItemRuleDraft) => Promise<void>;
	onRemove: (id: string) => Promise<void>;
}

/**
 * One rule on the Item Rules page, in one of two modes.
 *
 * Reading: the item names it matches, the filters it adds, and whether items
 * typed into the list get it too, with Edit and Delete.
 *
 * Editing: the whole rule at once, via [`ItemRuleForm`] — the spec's inline
 * edit, with the manual-additions toggle inside the form.
 */
export function ItemRuleCard({ rule, onSave, onRemove }: ItemRuleCardProps) {
	const [editing, setEditing] = useState(false);
	const [confirmingRemoval, setConfirmingRemoval] = useState(false);
	// Buttons are named after the rule's first trigger so a screen reader (and
	// the e2e suite) can tell one rule's Edit from another's.
	const name = rule.triggers[0] ?? "rule";

	async function save(draft: ItemRuleDraft) {
		await onSave(rule.id, draft);
		setEditing(false);
	}

	return (
		<Card data-testid="item-rule" data-rule-name={name}>
			{editing ? (
				<ItemRuleForm
					idPrefix={`rule-${rule.id}`}
					initial={rule}
					submitLabel="Save rule"
					onSubmit={save}
					onCancel={() => setEditing(false)}
				/>
			) : (
				<CardContent className="flex flex-col gap-3 p-4">
					<div className="flex flex-col gap-1">
						<span className="text-xs font-medium text-muted-foreground">
							When an item is called
						</span>
						<TermChips terms={rule.triggers} label="Item names" />
					</div>
					<div className="flex flex-col gap-1">
						<span className="text-xs font-medium text-muted-foreground">
							Add the filters
						</span>
						<TermChips terms={rule.filter_terms} label="Product filters" />
					</div>

					<div className="flex flex-wrap items-center justify-between gap-2">
						<span className="text-xs text-muted-foreground">
							{rule.apply_to_manual
								? "Voice and manual additions"
								: "Voice additions only"}
						</span>
						<div className="flex gap-2">
							{confirmingRemoval ? (
								<>
									<Button
										size="sm"
										variant="destructive"
										onClick={() => onRemove(rule.id)}
										aria-label={`Confirm deleting rule ${name}`}
									>
										Delete
									</Button>
									<Button
										size="sm"
										variant="ghost"
										onClick={() => setConfirmingRemoval(false)}
									>
										Keep
									</Button>
								</>
							) : (
								<>
									<Button
										size="sm"
										variant="outline"
										onClick={() => setEditing(true)}
										aria-label={`Edit rule ${name}`}
									>
										Edit
									</Button>
									<Button
										size="sm"
										variant="ghost"
										onClick={() => setConfirmingRemoval(true)}
										aria-label={`Delete rule ${name}`}
									>
										Delete
									</Button>
								</>
							)}
						</div>
					</div>
				</CardContent>
			)}
		</Card>
	);
}
