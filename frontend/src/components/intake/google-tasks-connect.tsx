import { useGoogleTasksSection } from "#/components/intake/google-tasks-section";
import { Button } from "#/components/ui/button";

/**
 * The first step: what the server still needs, or the button that sends the
 * household to Google's sign-in page. Once connected, says so.
 */
export function GoogleTasksConnect() {
	const { status, busy, connect } = useGoogleTasksSection();

	if (!status.configured) {
		return (
			<p className="text-sm text-muted-foreground">
				Not set up yet. The home server needs GOOGLE_CLIENT_ID and
				GOOGLE_CLIENT_SECRET — see "Google Tasks" in the setup guide.
			</p>
		);
	}
	if (!status.encryption_ready) {
		return (
			<p className="text-sm text-muted-foreground">
				Not set up yet. The home server needs CREDENTIAL_ENCRYPTION_KEY to keep
				the Google sign-in safe — see the setup guide, part 1.
			</p>
		);
	}
	if (status.connected) {
		return (
			<p className="text-sm">
				Connected
				{status.connected_at &&
					` since ${new Date(status.connected_at).toLocaleDateString()}`}
				.
			</p>
		);
	}
	return (
		<div className="flex flex-col gap-2">
			<p className="text-sm text-muted-foreground">
				Connect the Google account that has your grocery list.
			</p>
			<Button
				className="w-full sm:w-fit"
				disabled={busy !== null}
				onClick={connect}
			>
				Connect Google Tasks
			</Button>
		</div>
	);
}
