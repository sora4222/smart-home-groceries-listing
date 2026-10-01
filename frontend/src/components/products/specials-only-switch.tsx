import { useId } from "react";

import { Switch } from "#/components/ui/switch";

/**
 * "Specials only": hides products with no special or multibuy. A display
 * filter — it changes what is shown, not how anything is ordered or bought.
 */
export function SpecialsOnlySwitch({
	checked,
	onCheckedChange,
}: {
	checked: boolean;
	onCheckedChange: (checked: boolean) => void;
}) {
	const id = useId();
	return (
		<div className="flex items-center gap-2">
			<Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
			<label htmlFor={id} className="text-sm">
				Specials only
			</label>
		</div>
	);
}
