/**
 * The classifier's confidence in plain words, e.g. "85% sure", or null when
 * it never answered (it failed or timed out).
 */
export function confidenceLabel(confidence: number | null): string | null {
	if (confidence === null) return null;
	return `${Math.round(confidence * 100)}% sure`;
}
