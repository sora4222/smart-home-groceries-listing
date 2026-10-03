import { Badge } from "#/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import type { IntakeSettings } from "#/lib/api";

interface Row {
	name: string;
	state: string;
	ready: boolean;
	note: string;
}

/** The other ways in, and the item checker, as one row each. */
function rowsFor(settings: IntakeSettings): Row[] {
	return [
		{
			name: "Alexa",
			state: settings.alexa.configured ? "Set up" : "Not set up",
			ready: settings.alexa.configured,
			note: "Set up in the Alexa developer console — setup guide, part 4.",
		},
		{
			name: "Webhook",
			state: settings.webhook.configured ? "Set up" : "Not set up",
			ready: settings.webhook.configured,
			note: "For Home Assistant or IFTTT, using VOICE_WEBHOOK_SECRET.",
		},
		{
			name: "Item checker",
			state: settings.triage.provider,
			ready: settings.triage.provider !== "off",
			note: ["fake", "off"].includes(settings.triage.provider)
				? "Change INTAKE_LLM_PROVIDER in .env."
				: `Model: ${settings.triage.model}. Change INTAKE_LLM_PROVIDER in .env.`,
		},
		{
			name: "Google Keep",
			state: "Paused",
			ready: false,
			note: "Not built yet.",
		},
	];
}

/**
 * The intake channels set in the server's `.env` rather than here, so this
 * card only reads them out.
 */
export function OtherChannels({ settings }: { settings: IntakeSettings }) {
	return (
		<Card>
			<CardHeader>
				<CardTitle className="text-base">Other ways in</CardTitle>
			</CardHeader>
			<CardContent>
				<ul className="flex flex-col gap-3">
					{rowsFor(settings).map((row) => (
						<li key={row.name} className="flex flex-col gap-1">
							<div className="flex items-center gap-2">
								<span className="text-sm font-medium">{row.name}</span>
								<Badge variant={row.ready ? "secondary" : "outline"}>
									{row.state}
								</Badge>
							</div>
							<p className="text-xs text-muted-foreground">{row.note}</p>
						</li>
					))}
				</ul>
			</CardContent>
		</Card>
	);
}
