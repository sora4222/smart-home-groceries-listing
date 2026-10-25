/** Release/configuration security contracts, independent of host credentials or services. */
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, statSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { gzipSync } from 'node:zlib';
import { decodeBundle, digest, extractBundle, packBundle } from './bundle.mjs';
import { encodeConfiguration, mergeConfiguration } from './configuration.mjs';
import { startOptions } from './start-options.mjs';
import { unchangedIdentity } from './runtime.mjs';
import { postgresEnvironment } from './database.mjs';
import { parseEnv } from 'node:util';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertDataIdentity, recordDataIdentity } from './identity.mjs';

function fixture(run) {
  const directory = mkdtempSync(join(tmpdir(), 'grocery-release-test-'));
  try {
    for (const path of ['bin', 'frontend/.output/server', 'migrations']) mkdirSync(join(directory, path), { recursive: true });
    writeFileSync(join(directory, 'bin/grocery-backend'), '#!/bin/sh\nexit 0\n', { mode: 0o755 });
    writeFileSync(join(directory, 'frontend/.output/server/index.mjs'), 'console.log("fixture")');
    writeFileSync(join(directory, 'migrations/0001_initial.sql'), 'CREATE TABLE fixture(id int);');
    const metadata = { commit: 'a'.repeat(40), version: 'release-1.0.0', platform: 'linux-x64', python: '3.14' };
    run(directory, metadata);
  } finally { rmSync(directory, { recursive: true, force: true }); }
}

test('data-only bundle round trips runtime files, excluding root helpers and configuration', () => fixture((source, metadata) => {
  writeFileSync(join(source, '.env'), 'PRIVATE=never-package');
  mkdirSync(join(source, 'libexec'));
  writeFileSync(join(source, 'libexec/installer'), 'root code must not ship');
  const bundle = decodeBundle(packBundle(source, metadata));
  assert.equal(bundle.files.length, 3);
  const destination = source + '-extracted';
  try {
    extractBundle(bundle, destination);
    assert.match(readFileSync(join(destination, 'bin/grocery-backend'), 'utf8'), /exit 0/);
    assert.throws(() => extractBundle(bundle, destination));
  } finally { rmSync(destination, { recursive: true, force: true }); }
}));

test('generated Nitro chunk names with literal brackets are valid runtime paths', () => fixture((source, metadata) => {
  writeFileSync(join(source, 'frontend/.output/server/core+[...].mjs'), 'export {};');
  assert.equal(decodeBundle(packBundle(source, metadata)).files.length, 4);
}));

test('runtime remains readable/executable with a root-only deployment umask', () => fixture((source, metadata) => {
  const bundle = decodeBundle(packBundle(source, metadata));
  const destination = source + '-private-umask';
  const previous = process.umask(0o077);
  try {
    extractBundle(bundle, destination);
    assert.equal(statSync(join(destination, 'bin/grocery-backend')).mode & 0o777, 0o755);
    assert.equal(statSync(join(destination, 'bin')).mode & 0o777, 0o755);
    assert.equal(statSync(join(destination, 'frontend/.output/server/index.mjs')).mode & 0o777, 0o644);
  } finally { process.umask(previous); rmSync(destination, { recursive: true, force: true }); }
}));

test('bundle rejects traversal, installer replacement, duplicate paths and tampered bytes', () => fixture((source, metadata) => {
  const good = decodeBundle(packBundle(source, metadata));
  for (const path of ['../etc/passwd', '/etc/passwd', 'frontend/a/../../etc', 'libexec/grocery-env', 'frontend/.env', 'frontend//index']) {
    const bad = structuredClone(good); bad.files[0].path = path;
    assert.throws(() => decodeBundle(gzipSync(JSON.stringify(bad))));
  }
  const duplicate = structuredClone(good); duplicate.files.push(duplicate.files[0]);
  assert.throws(() => decodeBundle(gzipSync(JSON.stringify(duplicate))));
  const tamper = structuredClone(good); tamper.files[0].data = Buffer.from('changed').toString('base64');
  assert.throws(() => decodeBundle(gzipSync(JSON.stringify(tamper))));
  assert.throws(() => decodeBundle(Buffer.from('not gzip')));
}));

test('packaging rejects runtime links pointing outside the release', () => fixture((source, metadata) => {
  symlinkSync('/etc/passwd', join(source, 'frontend/passwords'));
  assert.throws(() => packBundle(source, metadata), /escapes/);
}));

test('startup preserves omitted values, accepts exports, and never stores bootstrap credentials', () => {
  const result = mergeConfiguration({ DATABASE_URL: 'old', CREDENTIAL_ENCRYPTION_KEY: 'keep', STORE_TIMEOUT_SECONDS: '12' },
    { STORE_TIMEOUT_SECONDS: '15' }, { STORE_TIMEOUT_SECONDS: '17', OP_SERVICE_ACCOUNT_TOKEN: 'do-not-store' });
  assert.deepEqual(result, { DATABASE_URL: 'old', CREDENTIAL_ENCRYPTION_KEY: 'keep', STORE_TIMEOUT_SECONDS: '17' });
  const values = { DATABASE_URL: 'postgres://user:pass@localhost/database', CLERK_SECRET_KEY: 'line1\nline2 "quoted"' };
  assert.deepEqual(parseEnv(encodeConfiguration(values)), values);
});

test('1Password is optional; secret arguments are not accepted', () => {
  assert.deepEqual(startOptions([], {}), { environmentId: undefined, token: undefined });
  assert.equal(startOptions(['--op-token-file', '/protected/token'], {}).tokenFile, '/protected/token');
  assert.equal(startOptions([], { OP_SERVICE_ACCOUNT_TOKEN: 'fixture' }).token, 'fixture');
  assert.throws(() => startOptions(['--op-token', 'secret'], {}));
  assert.throws(() => startOptions(['--op-environment-id', 'invalid/value'], {}));
});

test('updates cannot silently change database or credential encryption identity', () => {
  const before = { DATABASE_URL: 'postgres://user:old@localhost/db', CREDENTIAL_ENCRYPTION_KEY: 'key' };
  unchangedIdentity(before, before);
  unchangedIdentity(before, { ...before, DATABASE_URL: 'postgres://user:rotated@localhost/db' });
  assert.throws(() => unchangedIdentity(before, { ...before, DATABASE_URL: 'postgres://user:old@localhost/other' }));
  assert.throws(() => unchangedIdentity(before, { ...before, CREDENTIAL_ENCRYPTION_KEY: 'new' }));
});

test('database secrets go to the subprocess environment rather than command arguments', () => {
  const env = postgresEnvironment('postgresql://user:p%40ss@localhost:5432/example');
  assert.equal(env.PGPASSWORD, 'p@ss');
  assert.equal(env.PGDATABASE, 'example');
  assert.equal(env.OP_SERVICE_ACCOUNT_TOKEN, undefined);
  assert.throws(() => postgresEnvironment('https://example.invalid'));
});

test('retained identity permits password rotation and blocks data/key replacement', () => fixture((directory) => {
  const file = join(directory, 'identity.json');
  const settings = { DATABASE_URL: 'postgres://user:first@localhost/db', CREDENTIAL_ENCRYPTION_KEY: 'original-key' };
  recordDataIdentity(settings, file);
  assertDataIdentity({ ...settings, DATABASE_URL: 'postgres://user:rotated@localhost/db' }, file);
  assert.throws(() => assertDataIdentity({ ...settings, DATABASE_URL: 'postgres://user:first@localhost/empty' }, file));
  assert.throws(() => assertDataIdentity({ ...settings, CREDENTIAL_ENCRYPTION_KEY: 'wrong-key' }, file));
  assert.ok(!readFileSync(file, 'utf8').includes('original-key'));
}));

test('installed configuration CLI executes correctly through a control/release symlink', () => fixture((directory) => {
  const link = join(directory, 'resolver.mjs');
  symlinkSync(fileURLToPath(new URL('../../scripts/service-env.mjs', import.meta.url)), link);
  const result = spawnSync(process.execPath, [link, '--format', 'json', 'backend'], {
    input: 'DATABASE_URL=postgres://fixture:fixture@localhost/fixture\n', encoding: 'utf8', env: { PATH: process.env.PATH },
  });
  assert.equal(result.status, 0);
  assert.equal(JSON.parse(result.stdout).DATABASE_URL, 'postgres://fixture:fixture@localhost/fixture');
}));
