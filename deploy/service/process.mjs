/** Bounded subprocesses with secret-safe errors and explicit environments. */
import { spawnSync } from 'node:child_process';

export const systemEnvironment = { PATH: '/usr/local/bin:/usr/bin:/bin', HOME: '/root', GROCERY_ENV_FILE: '/dev/null' };

/** Run a fixed program; keep stdout private unless the caller explicitly uses it. */
export function command(program, args, options = {}) {
  const result = spawnSync(program, args, { env: systemEnvironment, cwd: '/', encoding: 'utf8', timeout: 120000,
    maxBuffer: 16 * 1024 * 1024, stdio: ['pipe', 'pipe', 'pipe'], ...options });
  if (result.error || result.status !== 0) throw new Error(program.split('/').pop() + ' failed; no configuration values were logged');
  return result.stdout;
}
