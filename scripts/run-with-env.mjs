/**
 * Run one command as an unprivileged user with a JSON environment from stdin.
 *
 *   grocery-env json backend | node run-with-env.mjs nobody CMD [ARGS...]
 *
 * Used by `make hm-install-server` to apply migrations with the new backend binary
 * before the running one is replaced. Secrets travel through the pipe, never
 * the command line, and the command gets no other variables (no `OP_*`).
 */
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { userInfo } from 'node:os';
import { pathToFileURL } from 'node:url';

/** uid and gid of a system user, read from /etc/passwd. */
export function lookupUser(name, passwd = readFileSync('/etc/passwd', 'utf8')) {
  const line = passwd.split('\n').find(entry => entry.split(':')[0] === name);
  if (!line) throw new Error(`No such user ${name}`);
  const [, , uid, gid] = line.split(':');
  return { uid: Number(uid), gid: Number(gid) };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [user, command, ...args] = process.argv.slice(2);
    if (!user || !command) throw new Error('Usage: run-with-env.mjs USER CMD [ARGS...] < env.json');
    const env = { PATH: '/usr/bin:/bin', ...JSON.parse(readFileSync(0, 'utf8')), GROCERY_ENV_FILE: '/dev/null' };
    const ids = userInfo().uid === 0 ? lookupUser(user) : {};
    const result = spawnSync(command, args, { env, stdio: ['ignore', 'inherit', 'inherit'], cwd: '/', ...ids });
    if (result.error) throw result.error;
    process.exitCode = result.status ?? 1;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
