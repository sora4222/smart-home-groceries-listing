import { useId } from "react";

import { Switch } from "#/components/ui/switch";

/**
 * "Show health checks": the container asks `/api/health` every few seconds,
 * which would bury everything else, so those rows are hidden by default.
 */
export function HealthChecksSwitch({
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
				Show health checks
			</label>
		</div>
	);
}
