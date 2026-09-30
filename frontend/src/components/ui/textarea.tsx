import type * as React from "react";

import { cn } from "#/lib/utils";

/** Shadcn-pattern multi-line field, used for an item's free-text note. */
function Textarea({ className, ...props }: React.ComponentProps<"textarea">) {
	return (
		<textarea
			data-slot="textarea"
			className={cn(
				"flex min-h-16 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors",
				"placeholder:text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring",
				"disabled:cursor-not-allowed disabled:opacity-50",
				className,
			)}
			{...props}
		/>
	);
}

export { Textarea };
