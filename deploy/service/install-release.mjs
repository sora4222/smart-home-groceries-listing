#!/usr/bin/env node
/** Fixed native installer: runtime files only, application-role migration, retained backups. */
import { chmodSync, existsSync, lstatSync, mkdirSync, readFileSync, renameSync, symlinkSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { decodeBundle, digest, extractBundle } from './bundle.mjs';
import { backupDatabase, inspectMigrations, provisionDatabase } from './database.mjs';
import { command, systemEnvironment } from './process.mjs';
import { runtimeSettings } from './runtime.mjs';

const prefix = '/opt/grocery';
const releases = '/opt/grocery-releases';
const fixed = '/usr/local/lib/grocery-service';

/** Run migrations without root, sudo, bootstrap tokens or root's environment. */
function migrate(directory, settings) {
  const [uid, gid] = command('id', ['grocery-migrate']).match(/uid=(\d+).*gid=(\d+)/).slice(1).map(Number);
  command(join(directory, 'bin/grocery-backend'), ['--migrate-only'], {
    uid, gid, env: { ...systemEnvironment, ...settings, GROCERY_ENV_FILE: '/dev/null' }, timeout: 300000,
  });
}

/** Health failures keep the database and retained release available for deliberate recovery. */
function health() {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      command('curl', ['--fail', '--silent', '--max-time', '5', 'http://127.0.0.1:8000/api/health']);
      command('curl', ['--fail', '--silent', '--max-time', '5', 'http://127.0.0.1:3000/']);
      return;
    } catch { command('sleep', ['1']); }
  }
  throw new Error('New release failed health checks; database backup and previous release retained');
}

/** Install data-only runtime bytes; no downloaded module or shell script runs as root. */
export function installRelease(path) {
  const bytes = readFileSync(path);
  const bundle = decodeBundle(bytes);
  if (bundle.platform !== process.platform + '-' + process.arch) throw new Error('Release platform mismatch');
  const python = command('/usr/bin/python3', ['-c', 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")']).trim();
  if (bundle.python !== python) throw new Error('Release Python version mismatch');
  mkdirSync(releases, { recursive: true, mode: 0o755 });
  mkdirSync('/var/backups/grocery', { recursive: true, mode: 0o700 });
  chmodSync('/var/backups/grocery', 0o700);
  const destination = join(releases, bundle.commit + '-' + digest(bytes).slice(0, 16));
  if (!existsSync(destination)) {
    extractBundle(bundle, destination);
    for (const [name, target] of [['bin/node', '/usr/bin/node'], ['lib', fixed + '/lib'], ['libexec', fixed + '/libexec']]) symlinkSync(target, join(destination, name));
    if (existsSync(join(destination, 'alexa-bridge/venv/bin'))) {
      for (const name of ['python', 'python3', 'python' + python]) symlinkSync('/usr/bin/python3', join(destination, 'alexa-bridge/venv/bin', name));
      symlinkSync('lib', join(destination, 'alexa-bridge/venv/lib64'));
    }
    writeFileSync(join(destination, 'release.json'), JSON.stringify({ commit: bundle.commit, version: bundle.version, digest: digest(bytes) }) + '\n', { flag: 'wx', mode: 0o644 });
  }
  const settings = runtimeSettings('backend');
  if (!settings.DATABASE_URL) throw new Error('Configure DATABASE_URL before installing');
  const existing = existsSync(prefix);
  if (!existing) provisionDatabase(settings.DATABASE_URL);
  inspectMigrations(join(destination, 'migrations'), settings.DATABASE_URL);
  const stopped = existing;
  let switched = false;
  let migrating = false;
  try {
    if (stopped) command('systemctl', ['stop', 'grocery.target']);
    backupDatabase('/var/backups/grocery', settings.DATABASE_URL, bundle.commit);
    migrating = true;
    migrate(destination, settings);
    // Preserve an old directory install intact, then replace a symlink atomically.
    if (existing && !lstatSync(prefix).isSymbolicLink()) renameSync(prefix, join(releases, 'legacy-' + Date.now()));
    const temporary = prefix + '.next-' + process.pid;
    symlinkSync(destination, temporary);
    renameSync(temporary, prefix);
    switched = true;
    const services = ['grocery-backend.service', 'grocery-frontend.service'];
    if (existsSync(join(destination, 'alexa-bridge/venv/bin/gunicorn'))) services.push('grocery-alexa-bridge.service');
    command('systemctl', ['enable', 'grocery.target', ...services]);
    command('systemctl', ['start', 'grocery.target']);
    health();
    console.log('Installed healthy release ' + bundle.version);
  } catch (error) {
    // Do not automatically start old code against a potentially changed schema.
    if (stopped && !migrating) command('systemctl', ['start', 'grocery.target']);
    else if (stopped && !switched) {
      console.error('Installation stopped before file switch; inspect migration/backup state before restarting');
    }
    throw error;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.getuid() !== 0 || process.argv.length !== 3) throw new Error('Usage: sudo grocery-install-release BUNDLE');
    installRelease(process.argv[2]);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
