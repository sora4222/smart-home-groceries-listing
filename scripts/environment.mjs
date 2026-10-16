/** Load dotenv configuration without executing shell code or printing values. */
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

/** Try the exported service account, then retry once through the desktop app. */
export function readSecret(reference, inherited) {
  const desktop = { ...inherited, OP_BIOMETRIC_UNLOCK_ENABLED: 'true' };
  for (const key of Object.keys(desktop)) {
    if (key === 'OP_SERVICE_ACCOUNT_TOKEN' || key.startsWith('OP_CONNECT_') || key === 'OP_SESSION' || key.startsWith('OP_SESSION_')) delete desktop[key];
  }
  const read = env => {
    const value = execFileSync('op', ['read', '--no-newline', reference], {
      env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
    });
    if (!value || value.startsWith('op://')) throw new Error('Unresolved 1Password value');
    return value;
  };
  if (inherited.OP_SERVICE_ACCOUNT_TOKEN) {
    const service = { ...desktop, OP_SERVICE_ACCOUNT_TOKEN: inherited.OP_SERVICE_ACCOUNT_TOKEN, OP_BIOMETRIC_UNLOCK_ENABLED: 'false' };
    delete service.OP_ACCOUNT;
    try { return read(service); }
    catch { /* A failed service-account read gets one desktop attempt. */ }
  }
  return read(desktop);
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
