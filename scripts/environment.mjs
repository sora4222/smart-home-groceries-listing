/** Load dotenv configuration without executing shell code or printing values. */
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';
import { withOnePassword } from './op-auth.mjs';
import { readEnvironmentVariables } from './op-environment.mjs';

/** Resolve one `op://` reference with the shared 1Password authentication policy. */
export function readSecret(reference, inherited) {
  return withOnePassword(inherited, env => {
    const value = execFileSync('op', ['read', '--no-newline', reference], {
      env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
    });
    if (!value || value.startsWith('op://')) throw new Error('Unresolved 1Password value');
    return value;
  });
}

/**
 * Merge .env, then the optional 1Password Environment, then exported overrides,
 * and resolve only the selected keys. This matches `op run` precedence for the
 * first two sources.
 */
export function loadEnvironment({
  file, required = true, inherited = process.env, accepts = () => true, resolve = readSecret,
  environmentId, readEnvironment = readEnvironmentVariables,
}) {
  let values = {};
  try { values = parseEnv(readFileSync(file, 'utf8')); }
  catch (error) {
    if (required || error.code !== 'ENOENT') throw new Error('Cannot read environment file');
  }
  const environment = environmentId ? readEnvironment(environmentId, inherited) : {};
  const merged = { ...values, ...environment, ...inherited };
  const result = {};
  for (const [key, raw] of Object.entries(merged)) {
    if (!accepts(key) || raw === undefined) continue;
    let value = raw;
    if (value.startsWith('op://')) {
      try { value = resolve(value, inherited); }
      catch { throw new Error(`Cannot load ${key} from 1Password; check CLI access and the reference`); }
      if (!value || value.startsWith('op://')) throw new Error(`1Password returned no resolved value for ${key}`);
    }
    result[key] = value;
  }
  return result;
}
