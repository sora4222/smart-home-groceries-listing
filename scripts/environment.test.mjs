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
    file: '/nonexistent/fixture.env', required: false,
    inherited: { CLERK_SECRET_KEY: 'op://Development/clerk/secret', DATABASE_URL: 'op://Development/missing/url', VITE_EXAMPLE: 'a$#@ b' },
    accepts: key => key.startsWith('CLERK_') || key.startsWith('VITE_'),
    resolve: reference => { assert.equal(reference, 'op://Development/clerk/secret'); return 'resolved-secret'; },
  });
  assert.deepEqual(result, { CLERK_SECRET_KEY: 'resolved-secret', VITE_EXAMPLE: 'a$#@ b' });
});

for (const failure of ['denied', 'empty', 'unresolved']) {
test(`service-account ${failure} reads retry with desktop authentication only`, () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-auth-fallback-'));
  try {
    writeFileSync(join(dir, 'op'), `#!/bin/sh
[ "$1" = read ] && [ "$2" = --no-newline ] || exit 2
[ -z "$OP_CONNECT_HOST$OP_CONNECT_TOKEN$OP_SESSION$OP_SESSION_fixture" ] || exit 3
if [ -n "$OP_SERVICE_ACCOUNT_TOKEN" ]; then
  [ "$OP_BIOMETRIC_UNLOCK_ENABLED" = false ] || exit 4
  printf 'service\n' >> "$AUTH_TRACE"
  case "$SERVICE_RESULT" in
    empty) exit 0 ;;
    unresolved) printf op://unresolved; exit 0 ;;
  esac
  echo sensitive-provider-output >&2
  exit 7
fi
[ "$OP_BIOMETRIC_UNLOCK_ENABLED" = true ] || exit 5
printf 'desktop\n' >> "$AUTH_TRACE"
printf desktop-secret
`, { mode: 0o700 });
    const inherited = {
      PATH: dir, CLERK_SECRET_KEY: 'op://Development/test/secret',
      OP_SERVICE_ACCOUNT_TOKEN: 'denied-fixture-token', OP_CONNECT_HOST: 'unrelated',
      OP_CONNECT_TOKEN: 'unrelated', OP_SESSION: 'stale', OP_SESSION_fixture: 'stale',
      OP_BIOMETRIC_UNLOCK_ENABLED: 'false', AUTH_TRACE: join(dir, 'trace'),
      SERVICE_RESULT: failure,
    };
    const result = loadEnvironment({ file: join(dir, '.env'), required: false, inherited });
    assert.equal(result.CLERK_SECRET_KEY, 'desktop-secret');
    assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), 'service\ndesktop\n');
    assert.equal(inherited.OP_SERVICE_ACCOUNT_TOKEN, 'denied-fixture-token');
  } finally { rmSync(dir, { recursive: true }); }
});
}

test('provider failures expose only the variable name', () => {
  assert.throws(() => loadEnvironment({ file: '/nonexistent/fixture.env', required: false, inherited: { CLERK_SECRET_KEY: 'op://private/reference/secret' }, resolve: () => { throw Error('sensitive provider output'); } }), { message: 'Cannot load CLERK_SECRET_KEY from 1Password; check CLI access and the reference' });
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

test('Compose startup selects Docker or external PostgreSQL and propagates Compose failures', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-startup-'));
  try {
    mkdirSync(join(dir, 'scripts'));
    copyFileSync('Makefile', join(dir, 'Makefile'));
    for (const name of ['environment.mjs', 'op-auth.mjs', 'op-environment.mjs', 'run-env.mjs', 'compose.mjs', 'dev-context.mjs', 'test-process.mjs', 'database-profiles.mjs']) {
      copyFileSync(`scripts/${name}`, join(dir, 'scripts', name));
    }
    writeFileSync(join(dir, 'docker'), '#!/bin/sh\nprintf "%s" "$COMPOSE_PROFILES"\nexit 7\n', { mode: 0o700 });
    const env = { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env') };
    writeFileSync(join(dir, '.env'), 'DATABASE_URL=postgres://fixture@db/grocery\nCOMPOSE_PROFILES=ollama\n');
    const docker = spawnSync('node', ['scripts/compose.mjs', 'up', '-d'], { cwd: dir, env, encoding: 'utf8' });
    assert.notEqual(docker.status, 0);
    assert.equal(docker.stdout, 'ollama,db');
    assert.equal(readFileSync(join(dir, '.env'), 'utf8'), 'DATABASE_URL=postgres://fixture@db/grocery\nCOMPOSE_PROFILES=ollama\n');
    writeFileSync(join(dir, '.env'), 'DATABASE_URL=postgres://fixture@host.docker.internal/grocery\nCOMPOSE_PROFILES=ollama,db\n');
    const external = spawnSync('node', ['scripts/compose.mjs', 'up', '-d'], { cwd: dir, env, encoding: 'utf8' });
    assert.notEqual(external.status, 0);
    assert.equal(external.stdout, 'ollama');
  } finally { rmSync(dir, { recursive: true }); }
});

test('backend tests select the disposable database runner without reading application secrets', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-test-command-'));
  try {
    mkdirSync(join(dir, 'scripts'));
    mkdirSync(join(dir, 'backend'));
    copyFileSync('Makefile', join(dir, 'Makefile'));
    copyFileSync('scripts/backend-tests.sh', join(dir, 'scripts/backend-tests.sh'));
    writeFileSync(join(dir, '.env'), 'POSTGRES_PASSWORD=op://Development/test/password\n');
    writeFileSync(join(dir, 'op'), '#!/bin/sh\necho unexpected-provider-access >&2\nexit 7\n', { mode: 0o700 });
    writeFileSync(join(dir, 'node'), '#!/bin/sh\nprintf "%s %s" "$1" "$2"\n', { mode: 0o700 });
    const result = spawnSync('make', ['test-backend'], {
      cwd: dir, env: { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env') }, encoding: 'utf8',
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, 'scripts/backend-tests.mjs --compact');
    assert.equal(result.stderr, '');
  } finally { rmSync(dir, { recursive: true }); }
});
