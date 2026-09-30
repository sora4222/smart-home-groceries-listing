import { useState } from "react";

import { Button } from "#/components/ui/button";
import { Card, CardContent } from "#/components/ui/card";

interface CommitBarProps {
	/** How many items are still under review. */
	activeCount: number;
	/** How many are already locked in for purchase. */
	committedCount: number;
	onCommit: () => Promise<void>;
	onRelease: () => Promise<void>;
}

/**
 * The line between reviewing the list and buying from it.
 *
 * Committing is the household saying "this is what we want" — after it, items
 * are read-only until the list is released again, so nothing changes underneath
 * an order being priced up.
 *
 * Both states can be true at once: an item added after a commit starts the next
 * list, so the locked banner and the commit button are shown together rather
 * than one hiding the other.
 */
export function CommitBar({
	activeCount,
	committedCount,
	onCommit,
	onRelease,
}: CommitBarProps) {
	const [busy, setBusy] = useState(false);

	async function run(action: () => Promise<void>) {
		if (busy) return;
		setBusy(true);
		try {
			await action();
		} finally {
			setBusy(false);
		}
	}

	return (
		<div className="flex flex-col gap-3">
			{committedCount > 0 && (
				<Card className="border-primary">
					<CardContent className="flex flex-wrap items-center justify-between gap-3 p-4">
						<p className="text-sm">
							<span className="font-medium">
								{committedCount} {committedCount === 1 ? "item" : "items"}{" "}
								locked in for purchase.
							</span>{" "}
							<span className="text-muted-foreground">
								Release the list to change anything.
							</span>
						</p>
						<Button
							variant="outline"
							disabled={busy}
							onClick={() => run(onRelease)}
						>
							{busy ? "Releasing…" : "Release for editing"}
						</Button>
					</CardContent>
				</Card>
			)}

			{activeCount > 0 && (
				<div className="flex justify-end">
					<Button
						className="w-full sm:w-auto"
						disabled={busy}
						onClick={() => run(onCommit)}
					>
						{busy
							? "Committing…"
							: `Ready to order (${activeCount} ${activeCount === 1 ? "item" : "items"})`}
					</Button>
				</div>
			)}
		</div>
	);
}
