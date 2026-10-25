/** Public startup settings shared by SSR and the browser; never serialize process.env. */
export const PUBLIC_KEYS = [
	"VITE_API_BASE_URL",
	"VITE_WS_URL",
	"VITE_AUTH_MODE",
	"VITE_CLERK_PUBLISHABLE_KEY",
] as const;

export type PublicConfig = Partial<
	Record<(typeof PUBLIC_KEYS)[number], string>
>;

declare global {
	interface Window {
		__GROCERY_PUBLIC_CONFIG__?: PublicConfig;
	}
}

/** Select exactly the browser-safe settings, retaining explicit empty overrides. */
export function selectPublicConfig(
	values: Record<string, unknown>,
): PublicConfig {
	return Object.fromEntries(
		PUBLIC_KEYS.filter((key) => typeof values[key] === "string").map((key) => [
			key,
			values[key],
		]),
	);
}

/** Runtime exports override build defaults, so a release is independent of site secrets. */
export function publicConfig(): PublicConfig {
	const defaults = selectPublicConfig(import.meta.env);
	if (import.meta.env.SSR) {
		return { ...defaults, ...selectPublicConfig(process.env) };
	}
	return { ...defaults, ...window.__GROCERY_PUBLIC_CONFIG__ };
}

/** Escape HTML delimiters so a public value cannot terminate the configuration script. */
export function publicConfigScript(values: Record<string, unknown>): string {
	const json = JSON.stringify(selectPublicConfig(values)).replace(
		/[<>&\u2028\u2029]/g,
		(char) => `\\u${char.charCodeAt(0).toString(16).padStart(4, "0")}`,
	);
	return `window.__GROCERY_PUBLIC_CONFIG__=${json};`;
}
