/** Persist explicit startup configuration without overwriting existing values by omission. */
import { chmodSync, existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { parseEnv } from 'node:util';
import { accepts, COMPONENTS } from '../../scripts/service-components.mjs';

/** Only application component settings may be persisted as runtime configuration. */
export const applicationKey = key => Object.values(COMPONENTS).some(spec => accepts(spec, key));

/** Read dotenv without shell evaluation; error messages never contain values. */
export function readConfiguration(path) {
  return existsSync(path) ? parseEnv(readFileSync(path, 'utf8')) : {};
}

/** Merge explicit file and exported values over retained local settings. */
export function mergeConfiguration(previous, supplied, inherited) {
  return { ...Object.fromEntries(Object.entries(previous).filter(([key]) => !key.startsWith('OP_'))),
    ...Object.fromEntries(Object.entries({ ...supplied, ...inherited })
      .filter(([key, value]) => applicationKey(key) && typeof value === 'string')) };
}

/** Replace a protected file atomically; never follow a caller-controlled output symlink. */
export function protectedWrite(path, content) {
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 });
  const temporary = path + '.' + process.pid + '.tmp';
  writeFileSync(temporary, content, { flag: 'wx', mode: 0o600 });
  renameSync(temporary, path);
  chmodSync(path, 0o600);
}

/** Encode dotenv strings with actual newlines, accepted by Node's parseEnv. */
export function encodeConfiguration(values) {
  return Object.entries(values).map(([key, value]) => {
    if (value.includes('\0') || (value.includes('"') && value.includes("'") && value.includes('`'))) throw new Error('Unsupported configuration encoding for ' + key);
    const quote = ['"', "'", '`'].find(candidate => !value.includes(candidate));
    return key + '=' + quote + value + quote;
  }).join('\n') + '\n';
}
