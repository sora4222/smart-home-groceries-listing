import { Button } from "#/components/ui/button";
import { Card, CardContent, CardFooter } from "#/components/ui/card";
import type { DuplicateItemDetail } from "#/lib/api";

interface DuplicatePromptProps {
	detail: DuplicateItemDetail;
	onMerge: () => void;
	onSeparate: () => void;
	onCancel: () => void;
}

/**
 * The question the spec's duplicate handling asks inline for manual additions:
 * "[Item] is already on the list. Add another or update the existing
 * quantity?" — shown when the backend answers 409 rather than guessing for the
 * user.
 */
export function DuplicatePrompt({
	detail,
	onMerge,
	onSeparate,
	onCancel,
}: DuplicatePromptProps) {
	return (
		<Card
			className="border-primary"
			role="alertdialog"
			aria-label="Already on the list"
		>
			<CardContent className="p-4 text-sm">
				<p>{detail.message}</p>
				<p className="mt-1 text-muted-foreground">
					On the list now: {detail.existing_item.name} ×
					{detail.existing_item.quantity}
				</p>
			</CardContent>
			<CardFooter className="flex-wrap p-4 pt-0">
				<Button size="sm" onClick={onMerge}>
					Update the quantity
				</Button>
				<Button size="sm" variant="outline" onClick={onSeparate}>
					Add a separate entry
				</Button>
				<Button size="sm" variant="ghost" onClick={onCancel}>
					Cancel
				</Button>
			</CardFooter>
		</Card>
	);
}
