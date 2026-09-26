import { useState } from "react";

import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
} from "#/components/ui/card";
import { cn } from "#/lib/utils";
import type { VoiceRequest } from "#/lib/api";

interface PendingRequestCardProps {
	request: VoiceRequest;
	dulled: boolean;
	onAccept: (id: string, name: string, quantity: number) => void;
	onReject: (id: string) => void;
}

/**
 * One card in the Pending Requests view (spec: "Google Home Integration >
 * Confirmation queue"). The item name and quantity are editable in case
 * Google misheard the request. A rejected card stays visible dulled with
 * an Accept button revealed on hover/focus, so an accidental reject is a
 * one-click undo rather than a lost item.
 */
export function PendingRequestCard({
	request,
	dulled,
	onAccept,
	onReject,
}: PendingRequestCardProps) {
	const [name, setName] = useState(request.parsed_name);
	const [quantity, setQuantity] = useState(request.parsed_quantity);

	return (
		<Card
			className={cn(
				"group transition-opacity",
				dulled && "opacity-40 hover:opacity-100 focus-within:opacity-100",
			)}
		>
			<CardHeader className="flex-row items-start justify-between gap-2">
				<div className="flex-1">
					<label className="sr-only" htmlFor={`name-${request.id}`}>
						Item name
					</label>
					<input
						id={`name-${request.id}`}
						value={name}
						onChange={(e) => setName(e.target.value)}
						className="w-full rounded-md border border-input bg-transparent px-2 py-1 text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring"
					/>
				</div>
				{dulled && <Badge variant="outline">Rejected</Badge>}
			</CardHeader>
			<CardContent className="flex items-center gap-3">
				<label
					className="text-sm text-muted-foreground"
					htmlFor={`qty-${request.id}`}
				>
					Qty
				</label>
				<input
					id={`qty-${request.id}`}
					type="number"
					min={1}
					max={999}
					value={quantity}
					onChange={(e) =>
						setQuantity(Math.max(1, Number(e.target.value) || 1))
					}
					className="w-20 rounded-md border border-input bg-transparent px-2 py-1 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
				/>
				<span className="text-xs text-muted-foreground">
					{new Date(request.created_at).toLocaleString()}
				</span>
			</CardContent>
			<CardFooter>
				<Button
					size="sm"
					onClick={() => onAccept(request.id, name, quantity)}
					aria-label={`Accept ${name}`}
				>
					Accept
				</Button>
				{!dulled && (
					<Button
						size="sm"
						variant="outline"
						onClick={() => onReject(request.id)}
						aria-label={`Reject ${name}`}
					>
						Reject
					</Button>
				)}
			</CardFooter>
		</Card>
	);
}
