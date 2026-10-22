/** One selected Compose context for startup, shutdown, and auxiliary services. */
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadEnvironment, selectedEnvironmentFile } from './environment.mjs';
import { databaseProfiles } from './database-profiles.mjs';
import { cancellation, runProcess } from './test-process.mjs';

/** Compose actions that need only Compose settings, never application secrets. */
export const READ_ONLY_ACTIONS = ['down', 'logs', 'ps', 'exec', 'stop'];

/** Run one Compose action with resolved settings; `overrides` win over loaded values. */
export async function runCompose({ action, args = [], files = [], overrides = {}, signal, log }) {
  const { file, required } = selectedEnvironmentFile();
  const readOnly = READ_ONLY_ACTIONS.includes(action);
  const env = loadEnvironment({ file, required, accepts: key => !readOnly || key.startsWith('COMPOSE_') });
  if (action === 'up') env.COMPOSE_PROFILES = databaseProfiles(env.DATABASE_URL, env.COMPOSE_PROFILES);
  const childEnv = { ...process.env, ...env, ...overrides };
  for (const key of Object.keys(childEnv)) if (key.startsWith('OP_')) delete childEnv[key];
  const fileArgs = files.flatMap(name => ['-f', name]);
  await runProcess('docker', ['compose', ...fileArgs, '--env-file', existsSync(file) ? file : '/dev/null', action, ...args], {
    cwd: fileURLToPath(new URL('../', import.meta.url)), env: childEnv, signal, log,
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [action, ...args] = process.argv.slice(2);
  try { await runCompose({ action, args, signal: cancellation() }); }
  catch (error) { console.error(error.message); process.exitCode ||= 1; }
}
