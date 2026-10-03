import { useGoogleTasksSection } from "#/components/intake/google-tasks-section";
import { Button } from "#/components/ui/button";

/**
 * How the last check went, "Check now", and "Disconnect".
 */
export function GoogleTasksActivity() {
	const { status, busy, checkNow, disconnect } = useGoogleTasksSection();

	return (
		<div className="flex flex-col gap-3 border-t border-border pt-4">
			<p className="text-sm text-muted-foreground">
				{status.last_polled_at
					? `Last checked ${new Date(status.last_polled_at).toLocaleString()}.`
					: "Not checked yet."}
			</p>
			{status.last_poll_error && (
				<p role="alert" className="text-sm text-destructive">
					{status.last_poll_error}
				</p>
			)}
			<div className="flex flex-col gap-2 sm:flex-row">
				<Button
					variant="secondary"
					disabled={busy !== null || !status.polling}
					onClick={checkNow}
				>
					Check now
				</Button>
				<Button variant="outline" disabled={busy !== null} onClick={disconnect}>
					Disconnect
				</Button>
			</div>
		</div>
	);
}
