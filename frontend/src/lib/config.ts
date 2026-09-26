/** Backend base URLs. Overridable via `.env` for non-default deployments. */
export const API_BASE_URL: string =
	import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8000";

export const WS_URL: string =
	import.meta.env.VITE_WS_URL ?? `${API_BASE_URL.replace(/^http/, "ws")}/ws`;
