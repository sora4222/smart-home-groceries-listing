/**
 * Resolve one production component's configuration and print it.
 *
 *   node service-env.mjs [--format systemd|json] [--token-file PATH] [--environment-id ID] COMPONENT < grocery.env
 *
 * The configuration is read from standard input, so the caller (root, in
 * `deploy/bin/grocery-env`) can open a file this process may not read. The
 * 1Password service-account token is read from `--token-file` rather than
 * the environment or the command line, and is never printed. An optional
 * 1Password Developer Environment overrides the standard-input values.
 */
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { loadEnvironment } from './environment.mjs';
import { accepts, component, renamed } from './service-components.mjs';
import { formatSystemdEnvironment } from './systemd-env.mjs';

/** Parse the command line into options. */
export function parseArguments(argv) {
  const options = { format: 'systemd', tokenFile: undefined, environmentId: undefined, name: undefined };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--format') options.format = argv[++index];
    else if (arg === '--token-file') options.tokenFile = argv[++index];
    else if (arg === '--environment-id') options.environmentId = argv[++index];
    else if (options.name === undefined) options.name = arg;
    else throw new Error(`Unexpected argument ${arg}`);
  }
  if (!options.name) throw new Error('Usage: service-env.mjs [--format systemd|json] [--token-file PATH] [--environment-id ID] COMPONENT');
  if (!['systemd', 'json'].includes(options.format)) throw new Error(`Unknown format ${options.format}`);
  return options;
}

/** Resolve the component's keys from `file` and the optional Environment, authorising 1Password with the token file if given. */
export function resolveComponent({ name, file, tokenFile, environmentId, inherited = process.env, resolve, readEnvironment }) {
  const spec = component(name);
  const env = { ...inherited };
  if (tokenFile) {
    env.OP_SERVICE_ACCOUNT_TOKEN = readFileSync(tokenFile, 'utf8').trim();
    if (!env.OP_SERVICE_ACCOUNT_TOKEN) throw new Error('The 1Password service-account token file is empty');
    env.GROCERY_OP_SERVICE_ONLY = '1';
  }
  const values = loadEnvironment({
    file, environmentId, inherited: env, accepts: key => accepts(spec, key),
    ...(resolve && { resolve }), ...(readEnvironment && { readEnvironment }),
  });
  return renamed(spec, values);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const options = parseArguments(process.argv.slice(2));
    // Exported variables are not configuration here: only standard input is.
    const inherited = { PATH: process.env.PATH, HOME: process.env.HOME, XDG_RUNTIME_DIR: process.env.XDG_RUNTIME_DIR };
    for (const key of Object.keys(inherited)) if (inherited[key] === undefined) delete inherited[key];
    const values = resolveComponent({ name: options.name, file: 0, tokenFile: options.tokenFile, environmentId: options.environmentId, inherited });
    process.stdout.write(options.format === 'json' ? JSON.stringify(values) : formatSystemdEnvironment(values));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
