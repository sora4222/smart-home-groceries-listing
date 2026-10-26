import { Link } from "@tanstack/react-router";

import { Button } from "#/components/ui/button";

/** "Select item": opens the page for choosing the item's product. */
export function SelectItemLink({
	itemId,
	itemName,
}: {
	itemId: string;
	itemName: string;
}) {
	return (
		<Button asChild size="sm" variant="outline">
			<Link
				to="/items/$itemId/select"
				params={{ itemId }}
				aria-label={`Select item for ${itemName}`}
			>
				Select item
			</Link>
		</Button>
	);
}
