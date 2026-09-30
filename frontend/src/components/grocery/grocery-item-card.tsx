import { Badge } from "#/components/ui/badge";
import { Card, CardContent, CardHeader } from "#/components/ui/card";
import type { GroceryItem } from "#/lib/api";

interface GroceryItemCardProps {
	item: GroceryItem;
}

/**
 * One row of the grocery list. Shows what the household will be buying: the
 * name, how many, where it came from, and any annotation left on it.
 */
export function GroceryItemCard({ item }: GroceryItemCardProps) {
	return (
		<Card data-testid="grocery-item">
			<CardHeader className="flex-row items-start justify-between gap-3 p-4">
				<span className="font-medium">{item.name}</span>
				<Badge variant="secondary" aria-label={`Quantity ${item.quantity}`}>
					×{item.quantity}
				</Badge>
			</CardHeader>
			<CardContent className="p-4 pt-0 text-xs text-muted-foreground">
				Added via {item.source === "voice" ? "voice" : "the web app"}
			</CardContent>
		</Card>
	);
}
