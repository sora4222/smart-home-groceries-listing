/** One selected Compose context for startup, shutdown, and auxiliary services. */
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadEnvironment } from './environment.mjs';
import { databaseProfiles } from './database-profiles.mjs';
import { cancellation, runProcess } from './test-process.mjs';

try {
  const [action, ...args] = process.argv.slice(2);
  const file = process.env.GROCERY_ENV_FILE ?? fileURLToPath(new URL('../.env', import.meta.url));
  const env = loadEnvironment({ file, required: process.env.GROCERY_ENV_FILE !== undefined, accepts: key => action !== 'down' || key.startsWith('COMPOSE_') });
  if (action === 'up') env.COMPOSE_PROFILES = databaseProfiles(env.DATABASE_URL, env.COMPOSE_PROFILES);
  const childEnv = { ...process.env, ...env };
  for (const key of Object.keys(childEnv)) if (key.startsWith('OP_')) delete childEnv[key];
  await runProcess('docker', ['compose', '--env-file', existsSync(file) ? file : '/dev/null', action, ...args], {
    cwd: fileURLToPath(new URL('../', import.meta.url)), env: childEnv, signal: cancellation(),
  });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
