/** Load dotenv configuration without executing shell code or printing values. */
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

/** Resolve one reference using desktop integration or OP_SERVICE_ACCOUNT_TOKEN. */
export function readSecret(reference, inherited) {
  return execFileSync('op', ['read', '--no-newline', reference], {
    env: inherited, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
  });
}

/** Merge .env with exported overrides, then resolve only the selected keys. */
export function loadEnvironment({ file, inherited = process.env, accepts = () => true, resolve = readSecret }) {
  let values = {};
  try { values = parseEnv(readFileSync(file, 'utf8')); }
  catch (error) {
    if (error.code !== 'ENOENT') throw new Error('Cannot read environment file');
  }
  const merged = { ...values, ...inherited };
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
