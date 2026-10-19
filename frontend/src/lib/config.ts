/**
 * Backend addresses for this build, from `VITE_API_BASE_URL` and
 * `VITE_WS_URL` in `.env`. Rules: `lib/backend-url.ts`.
 */
import { resolveBackendUrls } from "#/lib/backend-url";

const urls = resolveBackendUrls(import.meta.env);

export const API_BASE_URL: string = urls.apiBaseUrl;

export const WS_URL: string = urls.wsUrl;
