import type * as React from "react";

import { cn } from "#/lib/utils";

/**
 * Shadcn-pattern text input. Extend the class list here rather than passing
 * one-off styling from a feature component, so every field on the site shares
 * a focus ring and a border token.
 */
function Input({ className, type, ...props }: React.ComponentProps<"input">) {
	return (
		<input
			type={type}
			data-slot="input"
			className={cn(
				"flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors",
				"placeholder:text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring",
				"disabled:cursor-not-allowed disabled:opacity-50",
				className,
			)}
			{...props}
		/>
	);
}

export { Input };
