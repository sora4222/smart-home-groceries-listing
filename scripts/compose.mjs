/** One selected Compose context for startup, shutdown, and auxiliary services. */
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadEnvironment } from './environment.mjs';
import { databaseProfiles } from './database-profiles.mjs';
import { developmentContext } from './dev-context.mjs';
import { cancellation, runProcess } from './test-process.mjs';

try {
  const argv = process.argv.slice(2);
  const development = argv[0] === '--development';
  if (development) argv.shift();
  const [action, ...args] = argv;
  const file = process.env.GROCERY_ENV_FILE ?? fileURLToPath(new URL('../.env', import.meta.url));
  const env = loadEnvironment({ file, required: process.env.GROCERY_ENV_FILE !== undefined, accepts: key => !['down', 'logs', 'ps', 'exec', 'stop'].includes(action) || key.startsWith('COMPOSE_') || (development && ['WEB_PORT', 'BACKEND_PORT'].includes(key)) });
  if (action === 'up') env.COMPOSE_PROFILES = databaseProfiles(env.DATABASE_URL, env.COMPOSE_PROFILES);
  const selected = development ? developmentContext(env) : env;
  const childEnv = { ...process.env, ...selected };
  for (const key of Object.keys(childEnv)) if (key.startsWith('OP_')) delete childEnv[key];
  if (development && action === 'up' && args.includes('--wait')) console.log(`dev: http://localhost:${selected.WEB_PORT} (API http://localhost:${selected.BACKEND_PORT})`);
  const files = development ? ['-f', 'docker-compose.yml', '-f', 'docker-compose.dev.yml'] : [];
  await runProcess('docker', ['compose', ...files, '--env-file', existsSync(file) ? file : '/dev/null', action, ...args], {
    cwd: fileURLToPath(new URL('../', import.meta.url)), env: childEnv, signal: cancellation(),
  });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
