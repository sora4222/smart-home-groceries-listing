/** Build as the login user with frontend settings from the server's privileged resolver. */
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { cancellation, runProcess } from './test-process.mjs';
import { toolchainEnvironment } from './toolchain-environment.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
try {
  // sudo opens the root-only token; only resolved frontend settings cross this pipe.
  const result = spawnSync('sudo', [
    'env', `GROCERY_NODE=${process.execPath}`, `GROCERY_CONFIG=${root}.env`,
    `GROCERY_RESOLVER=${root}scripts/service-env.mjs`,
    'sh', `${root}deploy/bin/grocery-env`, 'json', 'frontend',
  ], { stdio: ['inherit', 'pipe', 'inherit'], encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error('Cannot resolve production frontend configuration; check sudo and the installed 1Password account');
  const env = toolchainEnvironment(process.env);
  let values;
  try { values = JSON.parse(result.stdout); }
  catch { throw new Error('Invalid frontend configuration from resolver'); }
  Object.assign(env, values, { GROCERY_ENV_FILE: '/dev/null' });
  await runProcess('pnpm', ['--dir', `${root}frontend`, 'run', '--silent', 'build'], { env, signal: cancellation() });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
