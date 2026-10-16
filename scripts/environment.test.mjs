/** Public configuration-loader contract; all secrets here are test fixtures. */
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, copyFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { loadEnvironment } from './environment.mjs';

test('loads literals without a secret provider and preserves exported overrides', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-env-'));
  try {
    const file = join(dir, '.env');
    writeFileSync(file, "DATABASE_URL='postgres://user:pw@db/grocery'\nCLERK_SECRET_KEY=op://Development/clerk/secret\n");
    const result = loadEnvironment({ file, inherited: { CLERK_SECRET_KEY: 'exported' }, resolve: () => { throw Error('unexpected provider access'); } });
    assert.equal(result.DATABASE_URL, 'postgres://user:pw@db/grocery');
    assert.equal(result.CLERK_SECRET_KEY, 'exported');
  } finally { rmSync(dir, { recursive: true }); }
});

test('startup command resolves every requested key with desktop or service-account authentication', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-env-'));
  try {
    const keys = ['DATABASE_URL', 'POSTGRES_USER', 'POSTGRES_PASSWORD', 'CLERK_SECRET_KEY', 'CLERK_PUBLISHABLE_KEY', 'VITE_CLERK_PUBLISHABLE_KEY', 'CLERK_JWKS_URL', 'ALEXA_SKILL_ID'];
    writeFileSync(join(dir, '.env'), keys.map(key => `${key}=op://Development/test/${key}`).join('\n'));
    writeFileSync(join(dir, 'op'), '#!/bin/sh\n[ "$1" = read ] && [ "$2" = --no-newline ] || exit 2\nprintf "%s-%s" "${OP_SERVICE_ACCOUNT_TOKEN:+service}" "${3##*/}"\n', { mode: 0o700 });
    for (const token of ['', 'fixture-token']) {
      const probe = `const keys=${JSON.stringify(keys)}; for (const key of keys) { if (process.env[key] !== '${token ? 'service' : ''}-' + key) process.exit(1); } if (process.env.OP_SERVICE_ACCOUNT_TOKEN) process.exit(2);`;
      const result = spawnSync(process.execPath, ['scripts/run-env.mjs', process.execPath, '-e', probe], {
        env: { PATH: dir, GROCERY_ENV_FILE: join(dir, '.env'), OP_SERVICE_ACCOUNT_TOKEN: token }, encoding: 'utf8',
      });
      assert.equal(result.status, 0, result.stderr);
    }
  } finally { rmSync(dir, { recursive: true }); }
});

test('application scope excludes unrelated provider failures and preserves literal punctuation', () => {
  const result = loadEnvironment({
    file: '/nonexistent/fixture.env',
    inherited: { CLERK_SECRET_KEY: 'op://Development/clerk/secret', DATABASE_URL: 'op://Development/missing/url', VITE_EXAMPLE: 'a$#@ b' },
    accepts: key => key.startsWith('CLERK_') || key.startsWith('VITE_'),
    resolve: reference => { assert.equal(reference, 'op://Development/clerk/secret'); return 'resolved-secret'; },
  });
  assert.deepEqual(result, { CLERK_SECRET_KEY: 'resolved-secret', VITE_EXAMPLE: 'a$#@ b' });
});

test('provider failures expose only the variable name', () => {
  assert.throws(() => loadEnvironment({ file: '/nonexistent/fixture.env', inherited: { CLERK_SECRET_KEY: 'op://private/reference/secret' }, resolve: () => { throw Error('sensitive provider output'); } }), { message: 'Cannot load CLERK_SECRET_KEY from 1Password; check CLI access and the reference' });
});

test('denied CLI access stops the command without exposing provider output or changing dotenv', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-denied-'));
  try {
    const content = 'CLERK_SECRET_KEY=op://Development/test/secret\n';
    writeFileSync(join(dir, '.env'), content);
    writeFileSync(join(dir, 'op'), '#!/bin/sh\necho provider-secret >&2\nexit 7\n', { mode: 0o700 });
    const result = spawnSync(process.execPath, ['scripts/run-env.mjs', process.execPath, '-e', 'console.log("application started")'], {
      env: { PATH: dir, GROCERY_ENV_FILE: join(dir, '.env'), OP_SERVICE_ACCOUNT_TOKEN: 'denied-fixture-token' }, encoding: 'utf8',
    });
    assert.equal(result.status, 1);
    assert.equal(result.stdout, '');
    assert.equal(result.stderr.trim(), 'Cannot load CLERK_SECRET_KEY from 1Password; check CLI access and the reference');
    assert.equal(readFileSync(join(dir, '.env'), 'utf8'), content);
  } finally { rmSync(dir, { recursive: true }); }
});

test('Compose passes resolved backend and Alexa settings without provider credentials', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-compose-'));
  try {
    copyFileSync('docker-compose.yml', join(dir, 'docker-compose.yml'));
    writeFileSync(join(dir, '.env'), 'COMPOSE_PROFILES=db\nDATABASE_URL=op://Development/test/DATABASE_URL\nCLERK_JWKS_URL=op://Development/test/CLERK_JWKS_URL\nALEXA_SKILL_ID=op://Development/test/ALEXA_SKILL_ID\nPOSTGRES_PASSWORD=op://Development/test/POSTGRES_PASSWORD\n');
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nprintf "resolved-%s" "${3##*/}"\n', { mode: 0o700 });
    const result = spawnSync(process.execPath, ['scripts/run-env.mjs', 'docker', 'compose', '--project-directory', dir, 'config', '--format', 'json'], {
      env: { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env'), OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token' }, encoding: 'utf8',
    });
    assert.equal(result.status, 0, result.stderr);
    const services = JSON.parse(result.stdout).services;
    assert.equal(services.backend.environment.DATABASE_URL, 'resolved-DATABASE_URL');
    assert.equal(services.backend.environment.CLERK_JWKS_URL, 'resolved-CLERK_JWKS_URL');
    assert.equal(services['alexa-bridge'].environment.ALEXA_SKILL_ID, 'resolved-ALEXA_SKILL_ID');
    assert.equal(services.db.environment.POSTGRES_PASSWORD, 'resolved-POSTGRES_PASSWORD');
    for (const service of Object.values(services)) assert.equal(service.environment?.OP_SERVICE_ACCOUNT_TOKEN, undefined);
  } finally { rmSync(dir, { recursive: true }); }
});

test('make up rejects an invalid database profile and propagates Compose failures', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-startup-'));
  try {
    mkdirSync(join(dir, 'scripts'));
    copyFileSync('Makefile', join(dir, 'Makefile'));
    for (const name of ['environment.mjs', 'run-env.mjs', 'compose-up.sh', 'check-db-profile.sh']) {
      copyFileSync(`scripts/${name}`, join(dir, 'scripts', name));
    }
    writeFileSync(join(dir, 'docker'), '#!/bin/sh\necho compose-called\nexit 7\n', { mode: 0o700 });
    const env = { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env') };
    writeFileSync(join(dir, '.env'), 'DATABASE_URL=postgres://fixture@db/grocery\nCOMPOSE_PROFILES=ollama\n');
    const rejected = spawnSync('make', ['up'], { cwd: dir, env, encoding: 'utf8' });
    assert.notEqual(rejected.status, 0);
    assert.equal(rejected.stdout, '');
    assert.match(rejected.stderr, /does not include db/);
    writeFileSync(join(dir, '.env'), 'DATABASE_URL=postgres://fixture@db/grocery\nCOMPOSE_PROFILES=db\n');
    const failed = spawnSync('make', ['up'], { cwd: dir, env, encoding: 'utf8' });
    assert.notEqual(failed.status, 0);
    assert.equal(failed.stdout.trim(), 'compose-called');
  } finally { rmSync(dir, { recursive: true }); }
});

test('backend test target stops when database credential resolution fails', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-test-command-'));
  try {
    mkdirSync(join(dir, 'scripts'));
    mkdirSync(join(dir, 'backend'));
    copyFileSync('Makefile', join(dir, 'Makefile'));
    for (const name of ['environment.mjs', 'run-env.mjs', 'test-database-url.sh']) {
      copyFileSync(`scripts/${name}`, join(dir, 'scripts', name));
    }
    copyFileSync('scripts/backend-tests.sh', join(dir, 'scripts/backend-tests.sh'));
    writeFileSync(join(dir, '.env'), 'POSTGRES_PASSWORD=op://Development/test/password\n');
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nexit 7\n', { mode: 0o700 });
    writeFileSync(join(dir, 'cargo'), '#!/bin/sh\necho cargo-started\n', { mode: 0o700 });
    const result = spawnSync('make', ['test-backend'], {
      cwd: dir, env: { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env') }, encoding: 'utf8',
    });
    assert.notEqual(result.status, 0);
    assert.doesNotMatch(result.stdout, /cargo-started/);
    assert.match(result.stderr, /Cannot load POSTGRES_PASSWORD/);
  } finally { rmSync(dir, { recursive: true }); }
});
