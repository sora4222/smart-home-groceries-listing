#!/usr/bin/env node
/** Configure unattended startup from exports/files, optionally backed by 1Password. */
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { resolveComponent } from '../../scripts/service-env.mjs';
import { encodeConfiguration, mergeConfiguration, protectedWrite, readConfiguration } from './configuration.mjs';
import { startOptions } from './start-options.mjs';
import { command, systemEnvironment } from './process.mjs';
import { installRelease } from './install-release.mjs';
import { unchangedIdentity } from './runtime.mjs';

/** Save only validated settings; omit 1Password integration entirely when unconfigured. */
export function configure(options, inherited = process.env, etc = '/etc/grocery', tokenPath = '/etc/1password/grocery.token') {
  const previous = readConfiguration(join(etc, 'grocery.env'));
  const supplied = options.config ? readConfiguration(options.config) : {};
  if (options.config && !existsSync(options.config)) throw new Error('Configuration file does not exist');
  const merged = mergeConfiguration(previous, supplied, inherited);
  const currentId = existsSync(join(etc, 'op-environment-id')) ? readFileSync(join(etc, 'op-environment-id'), 'utf8').trim() : undefined;
  const environmentId = options.environmentId || currentId;
  const existingToken = existsSync(tokenPath) ? tokenPath : (existsSync('/etc/1password/token') ? '/etc/1password/token' : undefined);
  const token = options.tokenFile ? readFileSync(options.tokenFile, 'utf8').trim() : options.token;
  if ((options.tokenFile || options.token !== undefined) && !token?.trim()) throw new Error('Empty 1Password token');
  const temporary = mkdtempSync('/run/grocery-config.');
  try {
    const candidate = join(temporary, 'candidate.env');
    protectedWrite(candidate, encodeConfiguration(merged));
    let candidateToken = existingToken;
    if (token) { candidateToken = join(temporary, 'token'); protectedWrite(candidateToken, token.trim() + '\n'); }
    if (environmentId && !candidateToken) throw new Error('A 1Password Environment needs a service-account token');
    const resolve = (name, file, tokenFile, id) => resolveComponent({ name, file, tokenFile, environmentId: id, inherited: systemEnvironment });
    const next = resolve('backend', candidate, candidateToken, environmentId);
    if (!next.DATABASE_URL) throw new Error('DATABASE_URL is required');
    if (Object.keys(previous).length) {
      const before = resolve('backend', join(etc, 'grocery.env'), existingToken, currentId);
      unchangedIdentity(before, next);
    }
    resolve('frontend', candidate, candidateToken, environmentId);
    protectedWrite(join(etc, 'grocery.env'), encodeConfiguration(merged));
    if (token) protectedWrite(tokenPath, token.trim() + '\n');
    if (environmentId) protectedWrite(join(etc, 'op-environment-id'), environmentId + '\n');
  } finally { rmSync(temporary, { recursive: true, force: true }); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.getuid() !== 0) throw new Error('Run the startup installer with sudo');
    const options = startOptions(process.argv.slice(2));
    configure(options);
    delete process.env.OP_SERVICE_ACCOUNT_TOKEN;
    if (options.bundle) installRelease(options.bundle);
    else if (!existsSync('/opt/grocery/bin/grocery-backend')) throw new Error('Supply --bundle for a first installation');
    command('systemctl', ['enable', 'grocery.target', 'grocery-backend.service', 'grocery-frontend.service']);
    command('systemctl', ['restart', 'grocery.target']);
    command('systemctl', ['is-active', '--quiet', 'grocery-backend', 'grocery-frontend']);
    console.log('Grocery service configured, running and enabled at boot');
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
