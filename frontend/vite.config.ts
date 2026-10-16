import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import { loadEnvironment } from "../scripts/environment.mjs";

/**
 * The repository root, where the one `.env` lives (`docs/human-setup.md`).
 * `VITE_*` values in it reach the browser; nothing else does.
 */
const ENV_DIR = "..";

/**
 * Resolve frontend configuration before Vite or Clerk reads it. Backend and
 * tunnel credentials are outside this process's configuration boundary.
 */
function loadFrontendEnvironment(): void {
	Object.assign(
		process.env,
		loadEnvironment({
			file: process.env.GROCERY_ENV_FILE ?? new URL("../.env", import.meta.url),
			accepts: (key) => key.startsWith("CLERK_") || key.startsWith("VITE_"),
		}),
	);
}

const config = defineConfig(() => {
	loadFrontendEnvironment();
	return {
		envDir: ENV_DIR,
		resolve: { tsconfigPaths: true },
		server: {
			port: 3000,
		},
		plugins: [devtools(), tailwindcss(), tanstackStart(), viteReact(), nitro()],
	};
});

export default config;
