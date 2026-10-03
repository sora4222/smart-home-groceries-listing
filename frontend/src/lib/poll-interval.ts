/** How often the Google Tasks list can be checked, in words. */
export const POLL_CHOICES: ReadonlyArray<{ seconds: number; label: string }> = [
	{ seconds: 30, label: "Every 30 seconds" },
	{ seconds: 60, label: "Every minute" },
	{ seconds: 300, label: "Every 5 minutes" },
	{ seconds: 900, label: "Every 15 minutes" },
	{ seconds: 3600, label: "Every hour" },
];

/** "Every minute" for 60; a plain count for a gap not in the list. */
export function pollIntervalLabel(seconds: number): string {
	return (
		POLL_CHOICES.find((choice) => choice.seconds === seconds)?.label ??
		`Every ${seconds} seconds`
	);
}
