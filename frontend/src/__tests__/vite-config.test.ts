import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import config from "../../vite.config";

const KEYS = [
	"GROCERY_ENV_FILE",
	"VITE_FIXTURE_URL",
	"CLERK_FIXTURE_KEY",
	"SERVER_API_BASE_URL",
	"DATABASE_URL",
];
const saved = Object.fromEntries(KEYS.map((key) => [key, process.env[key]]));

describe("the Vite configuration", () => {
	afterEach(() => {
		for (const key of KEYS) {
			if (saved[key] === undefined) delete process.env[key];
			else process.env[key] = saved[key];
		}
	});

	it("loads only the frontend component's settings from the selected file", () => {
		const dir = mkdtempSync(join(tmpdir(), "grocery-vite-config-"));
		try {
			const file = join(dir, "selected.env");
			writeFileSync(
				file,
				"VITE_FIXTURE_URL=http://api.test\nCLERK_FIXTURE_KEY=clerk\nSERVER_API_BASE_URL=http://backend:8000\nDATABASE_URL=postgres://db/grocery\n",
			);
			process.env.GROCERY_ENV_FILE = file;
			const resolved = (config as unknown as () => { envDir: unknown })();

			expect(resolved.envDir).toBe(false);
			expect(process.env.VITE_FIXTURE_URL).toBe("http://api.test");
			expect(process.env.CLERK_FIXTURE_KEY).toBe("clerk");
			expect(process.env.SERVER_API_BASE_URL).toBe("http://backend:8000");
			expect(process.env.DATABASE_URL).toBe(saved.DATABASE_URL);
		} finally {
			rmSync(dir, { recursive: true });
		}
	});
});
