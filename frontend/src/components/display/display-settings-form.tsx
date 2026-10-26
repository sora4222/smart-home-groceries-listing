import { useId } from "react";

import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import type { DisplaySettings, ImageSize } from "#/lib/display-settings";

const IMAGE_SIZES: ReadonlyArray<{ value: ImageSize; label: string }> = [
	{ value: "sm", label: "Small" },
	{ value: "md", label: "Medium" },
	{ value: "lg", label: "Large" },
];

/** The Select item list's look: picture size and what prices to show. */
export function DisplaySettingsForm({
	value,
	onChange,
}: {
	value: DisplaySettings;
	onChange: (next: DisplaySettings) => void;
}) {
	const id = useId();
	return (
		<div className="flex flex-col gap-4">
			<div className="flex items-center gap-3">
				<label htmlFor={`${id}-size`} className="text-sm">
					Picture size
				</label>
				<Select
					value={value.imageSize}
					onValueChange={(size) =>
						onChange({ ...value, imageSize: size as ImageSize })
					}
				>
					<SelectTrigger id={`${id}-size`} className="w-32">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{IMAGE_SIZES.map((size) => (
							<SelectItem key={size.value} value={size.value}>
								{size.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			<SettingSwitch
				id={`${id}-was`}
				label="Do not display original price"
				checked={!value.showWasPrice}
				onCheckedChange={(hide) => onChange({ ...value, showWasPrice: !hide })}
			/>
			<SettingSwitch
				id={`${id}-unavailable`}
				label="Display unavailable items"
				checked={value.showUnavailable}
				onCheckedChange={(show) =>
					onChange({ ...value, showUnavailable: show })
				}
			/>
		</div>
	);
}

/** One labelled on/off setting. */
function SettingSwitch({
	id,
	label,
	checked,
	onCheckedChange,
}: {
	id: string;
	label: string;
	checked: boolean;
	onCheckedChange: (checked: boolean) => void;
}) {
	return (
		<div className="flex items-center gap-3">
			<Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
			<label htmlFor={id} className="text-sm">
				{label}
			</label>
		</div>
	);
}
