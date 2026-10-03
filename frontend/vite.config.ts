import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig, loadEnv } from "vite";

/**
 * The repository root, where the one `.env` lives (`docs/human-setup.md`).
 * `VITE_*` values in it reach the browser; nothing else does.
 */
const ENV_DIR = "..";

/**
 * Clerk's server middleware reads `CLERK_SECRET_KEY` from the process
 * environment, which Vite does not fill from `.env`. Copy it across for the
 * dev server, without overriding a value the shell already set.
 */
function shareClerkSecretWithServer(mode: string): void {
	const env = loadEnv(mode, ENV_DIR, "CLERK_");
	if (!process.env.CLERK_SECRET_KEY && env.CLERK_SECRET_KEY) {
		process.env.CLERK_SECRET_KEY = env.CLERK_SECRET_KEY;
	}
}

const config = defineConfig(({ mode }) => {
	shareClerkSecretWithServer(mode);
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
