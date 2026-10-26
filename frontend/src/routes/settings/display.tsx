import { createFileRoute } from "@tanstack/react-router";
import { toast } from "sonner";

import { DisplaySettingsForm } from "#/components/display/display-settings-form";
import { useDisplaySettings } from "#/hooks/useDisplaySettings";
import { type DisplaySettings, saveDisplay } from "#/lib/display-settings";

export const Route = createFileRoute("/settings/display")({
	component: DisplayPage,
});

/**
 * Settings › Display — how the Select item list looks. Kept in this browser
 * only. Every change shows a toast with Undo.
 */
function DisplayPage() {
	const settings = useDisplaySettings();

	function change(next: DisplaySettings) {
		const previous = settings;
		saveDisplay(next);
		toast.success("Display saved", {
			action: { label: "Undo", onClick: () => saveDisplay(previous) },
		});
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Display</h1>
				<p className="text-sm text-muted-foreground">
					How products look when you select an item. Saved in this browser.
				</p>
			</div>
			<DisplaySettingsForm value={settings} onChange={change} />
		</div>
	);
}
