/** Run Tauri with only desktop configuration and a worktree-specific dev identity. */
import { realpathSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { desktopContext, desktopEnvironment, desktopServerUrl } from './desktop-context.mjs';
import { loadEnvironment } from './environment.mjs';
import { cancellation, runProcess } from './test-process.mjs';

try {
  const [action, ...args] = process.argv.slice(2);
  const root = realpathSync(fileURLToPath(new URL('..', import.meta.url)));
  const context = desktopContext(root, action);
  const values = loadEnvironment({
    file: process.env.GROCERY_ENV_FILE ?? `${root}/.env`,
    required: process.env.GROCERY_ENV_FILE !== undefined,
    accepts: key => key === 'GROCERY_DESKTOP_SERVER_URL',
  });
  values.GROCERY_DESKTOP_SERVER_URL = desktopServerUrl(values.GROCERY_DESKTOP_SERVER_URL);
  await runProcess('pnpm', ['exec', 'tauri', action,
    ...(context ? ['--config', JSON.stringify(context)] : []), ...args], {
    cwd: `${root}/desktop`, env: desktopEnvironment({ ...process.env, ...values }),
    signal: cancellation(),
  });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
