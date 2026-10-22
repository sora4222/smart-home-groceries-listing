/** Frontend builds use caller configuration without sudo or installed server credentials. */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

for (const scenario of ['literal', 'human', 'service', 'desktop', 'fallback', 'denied', 'build-failure', 'environment', 'environment-denied', 'exports', 'explicit-file', 'missing-file']) {
  test(`unprivileged frontend build: ${scenario}`, () => {
    const dir = mkdtempSync(join(tmpdir(), 'grocery-build-auth-'));
    try {
      for (const name of ['scripts', 'deploy', 'frontend/.output', 'path']) mkdirSync(join(dir, name), { recursive: true });
      copyFileSync('Makefile', join(dir, 'Makefile'));
      copyFileSync('deploy/build-component.sh', join(dir, 'deploy/build-component.sh'));
      for (const name of ['command.mjs', 'build-frontend.mjs', 'environment.mjs', 'op-auth.mjs', 'op-environment.mjs', 'service-components.mjs', 'test-process.mjs', 'toolchain-environment.mjs']) {
        copyFileSync(`scripts/${name}`, join(dir, 'scripts', name));
      }
      const usesReference = ['service', 'desktop', 'fallback', 'denied'].includes(scenario);
      const usesEnvironment = ['environment', 'environment-denied'].includes(scenario);
      const envFile = join(dir, scenario === 'explicit-file' ? 'selected.env' : '.env');
      if (!['exports', 'environment', 'missing-file'].includes(scenario)) {
        writeFileSync(envFile, `CLERK_JWKS_URL=${usesReference ? 'op://Development/fixture/jwks' : 'resolved-jwks'}\nDATABASE_URL=op://unreadable/backend\nVITE_API_BASE_URL=http://localhost:8000\n`);
      }
      if (scenario === 'explicit-file') writeFileSync(join(dir, '.env'), 'CLERK_JWKS_URL=op://must-not-read/default\n');
      writeFileSync(join(dir, 'path/sudo'), `#!/bin/sh
printf called > '${dir}/sudo-called'
exit 99
`, { mode: 0o700 });
      writeFileSync(join(dir, 'path/op'), `#!/bin/sh
[ -z "$OP_CONNECT_HOST$OP_CONNECT_TOKEN$OP_SESSION_fixture" ] || exit 2
if [ -n "$OP_SERVICE_ACCOUNT_TOKEN" ]; then
  [ "$OP_SERVICE_ACCOUNT_TOKEN" = fixture-token ] && [ "$OP_BIOMETRIC_UNLOCK_ENABLED" = false ] && [ -z "$OP_ACCOUNT" ] || exit 3
  printf 'service\n' >> '${dir}/trace'
  [ '${scenario}' != fallback ] || exit 7
else
  [ "$OP_BIOMETRIC_UNLOCK_ENABLED" = true ] || exit 4
  printf 'desktop\n' >> '${dir}/trace'
fi
[ '${scenario}' != denied ] && [ '${scenario}' != environment-denied ] || { echo provider-private-error >&2; exit 7; }
if [ "$1" = run ]; then
  [ "$3" = envfixtureid ] || exit 5
  shift 5
  exec env -u PWD -u OLDPWD -u SHLVL -u _ CLERK_JWKS_URL=resolved-jwks VITE_API_BASE_URL=https://environment.example:8101 DATABASE_URL=environment-private "$@"
fi
[ "$1" = read ] && [ "$3" = op://Development/fixture/jwks ] || exit 6
printf resolved-jwks
`, { mode: 0o700 });
      const expectedUrl = scenario === 'environment' ? 'https://environment.example:8101' : scenario === 'exports' ? 'https://export.example' : 'http://localhost:8000';
      writeFileSync(join(dir, 'path/pnpm'), `#!/bin/sh
[ "$3" != install ] || exit 0
[ "$CLERK_JWKS_URL" = resolved-jwks ] || exit 10
[ "$VITE_API_BASE_URL" = '${expectedUrl}' ] || exit 11
[ "$GROCERY_ENV_FILE" = /dev/null ] || exit 12
[ -z "$OP_SERVICE_ACCOUNT_TOKEN$OP_CONNECT_TOKEN$OP_SESSION_fixture$OP_ENVIRONMENT_ID$DATABASE_URL$STORE_TAB_SECRET" ] || exit 13
printf build-started
[ '${scenario}' != build-failure ]
`, { mode: 0o700 });
      const result = spawnSync('make', ['--no-print-directory', '-C', dir, 'build-frontend', `human=${scenario === 'human' ? 1 : 0}`], {
        encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
        env: { PATH: `${dir}/path:${process.env.PATH}`, HOME: process.env.HOME,
          ...(process.env.TMPDIR && { TMPDIR: process.env.TMPDIR }),
          OP_CONNECT_TOKEN: 'stale', OP_SESSION_fixture: 'stale',
          DATABASE_URL: 'backend-private', STORE_TAB_SECRET: 'store-private',
          ...(['service', 'fallback'].includes(scenario) && { OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token' }),
          ...(usesEnvironment && { OP_ENVIRONMENT_ID: 'envfixtureid' }),
          ...(['explicit-file', 'missing-file'].includes(scenario) && { GROCERY_ENV_FILE: scenario === 'missing-file' ? join(dir, 'absent.env') : envFile }),
          ...(['exports', 'missing-file'].includes(scenario) && { CLERK_JWKS_URL: 'resolved-jwks', VITE_API_BASE_URL: 'https://export.example' }),
        },
      });
      const succeeds = !['denied', 'build-failure', 'environment-denied', 'missing-file'].includes(scenario);
      assert.equal(existsSync(join(dir, 'sudo-called')), false, 'build must never call sudo');
      assert.equal(result.status === 0, succeeds, result.stderr);
      assert.equal(result.stdout, succeeds ? (scenario === 'human' ? 'build-startedbuilt build/frontend\n' : 'build-frontend: passed\n') : '', result.stderr);
      if (scenario === 'build-failure') assert.match(result.stderr, /build-started/);
      assert.doesNotMatch(result.stdout + result.stderr, /fixture-token|provider-private-error|backend-private|store-private|environment-private/);
      if (usesReference || usesEnvironment) {
        const attempts = scenario === 'service' ? 'service\n' : scenario === 'fallback' ? 'service\ndesktop\n' : 'desktop\n';
        assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), attempts);
      } else assert.equal(existsSync(join(dir, 'trace')), false);
      assert.equal(existsSync(join(dir, 'build/frontend/.output')), succeeds);
    } finally { rmSync(dir, { recursive: true }); }
  });
}
