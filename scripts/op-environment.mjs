/** Read the variables a 1Password Developer Environment adds, without printing them. */
import { execFileSync } from 'node:child_process';
import { withOnePassword } from './op-auth.mjs';

/** Prints the child's environment as JSON; `op run` adds the Environment's variables to it. */
const PRINT_ENVIRONMENT = 'process.stdout.write(JSON.stringify(process.env))';

/** Return only the variables `op run --environment` adds to the given environment. */
export function readEnvironmentVariables(environmentId, inherited) {
  if (!/^[a-z0-9]+$/i.test(environmentId)) throw new Error('Invalid 1Password Environment ID');
  const read = env => {
    const output = execFileSync('op', ['run', '--environment', environmentId, '--no-masking', '--', process.execPath, '-e', PRINT_ENVIRONMENT], {
      env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
    });
    const added = {};
    for (const [key, value] of Object.entries(JSON.parse(output))) {
      if (!Object.hasOwn(env, key) && !key.startsWith('OP_')) added[key] = value;
    }
    return added;
  };
  try { return withOnePassword(inherited, read); }
  catch { throw new Error('Cannot read the 1Password Environment; check CLI access and the Environment ID'); }
}
