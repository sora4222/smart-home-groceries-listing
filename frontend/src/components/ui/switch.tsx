import type * as React from "react";

import { cn } from "#/lib/utils";

export interface SwitchProps
	extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onChange"> {
	checked: boolean;
	onCheckedChange: (checked: boolean) => void;
}

/**
 * Shadcn-pattern on/off switch, hand-built as a `role="switch"` button so it
 * needs no extra Radix package. Space and Enter toggle it, as for any button;
 * label it with a `<label htmlFor>` or `aria-label`.
 */
function Switch({
	checked,
	onCheckedChange,
	className,
	...props
}: SwitchProps) {
	return (
		<button
			type="button"
			role="switch"
			aria-checked={checked}
			data-slot="switch"
			data-state={checked ? "checked" : "unchecked"}
			onClick={() => onCheckedChange(!checked)}
			className={cn(
				"inline-flex h-5 w-9 shrink-0 items-center rounded-full border border-transparent transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
				checked ? "bg-primary" : "bg-input",
				className,
			)}
			{...props}
		>
			<span
				data-slot="switch-thumb"
				className={cn(
					"pointer-events-none block size-4 rounded-full bg-background shadow-sm transition-transform",
					checked ? "translate-x-4" : "translate-x-0",
				)}
			/>
		</button>
	);
}

export { Switch };
