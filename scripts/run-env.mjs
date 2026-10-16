/** Run a command with resolved configuration; leave .env unchanged. */
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { loadEnvironment } from './environment.mjs';

try {
  const argv = process.argv.slice(2);
  const prefixes = argv[0] === '--prefix' ? (argv.splice(0, 2)[1] ?? '').split(',') : null;
  const [command, ...args] = argv;
  if (!command) throw new Error('Usage: node scripts/run-env.mjs COMMAND [ARGS...]');
  const env = loadEnvironment({
    file: process.env.GROCERY_ENV_FILE ?? fileURLToPath(new URL('../.env', import.meta.url)),
    accepts: key => !prefixes || prefixes.some(prefix => key.startsWith(prefix)),
  });
  const childEnv = { ...process.env, ...env };
  // These credentials authorize the loader, not the application or containers.
  for (const key of Object.keys(childEnv)) {
    if (key.startsWith('OP_')) delete childEnv[key];
  }
  const child = spawn(command, args, { env: childEnv, stdio: 'inherit' });
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
  child.on('error', () => { console.error('Cannot start the configured command'); process.exitCode = 1; });
  child.on('exit', (code, signal) => { process.exitCode = code ?? (signal === 'SIGINT' ? 130 : 143); });
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
