import type { IntakeSource } from "#/lib/api";

const labels: Record<IntakeSource, string> = {
	alexa: "Alexa",
	webhook: "Webhook",
};

/** The name an intake channel is shown under, e.g. "Alexa". */
export function intakeSourceLabel(source: IntakeSource): string {
	return labels[source];
}
