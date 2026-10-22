/** Production build authentication without root, real credentials or application builds. */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

for (const scenario of ['success', 'denied', 'empty-token', 'shared-token', 'build-failure', 'sudo-failure']) {
  test(`frontend build uses installed service-account policy: ${scenario}`, () => {
    const dir = mkdtempSync(join(tmpdir(), 'grocery-build-auth-'));
    try {
      for (const name of ['scripts', 'deploy/bin', 'frontend', 'path', 'etc']) mkdirSync(join(dir, name), { recursive: true });
      for (const name of ['build-frontend.mjs', 'environment.mjs', 'service-env.mjs', 'service-components.mjs', 'systemd-env.mjs', 'test-process.mjs', 'toolchain-environment.mjs']) {
        copyFileSync(`scripts/${name}`, join(dir, 'scripts', name));
      }
      copyFileSync('deploy/bin/grocery-env', join(dir, 'deploy/bin/grocery-env'));
      writeFileSync(join(dir, '.env'), 'CLERK_JWKS_URL=op://Development/grocery-list-information/Clerk/jwks-url\nDATABASE_URL=op://unreadable/backend\nVITE_API_BASE_URL=http://localhost:8000\n');
      writeFileSync(join(dir, 'etc/shared.token'), 'fixture-shared');
      if (scenario !== 'shared-token') writeFileSync(join(dir, 'etc/app.token'), scenario === 'empty-token' ? '  \n' : 'fixture-app');
      writeFileSync(join(dir, 'path/sudo'), `#!/bin/sh
[ "$SCENARIO" != sudo-failure ] || exit 1
exec "$@"
`, { mode: 0o700 });
      writeFileSync(join(dir, 'path/op'), `#!/bin/sh
printf 'service-attempt\n' >> '${dir}/trace'
[ "$OP_BIOMETRIC_UNLOCK_ENABLED" = false ] || exit 2
[ -z "$OP_CONNECT_HOST$OP_CONNECT_TOKEN$OP_SESSION_fixture$OP_ACCOUNT" ] || exit 3
[ "$OP_SERVICE_ACCOUNT_TOKEN" = ${scenario === 'shared-token' ? 'fixture-shared' : 'fixture-app'} ] || exit 4
[ "$3" = op://Development/grocery-list-information/Clerk/jwks-url ] || exit 5
[ ! -f '${dir}/deny' ] || { echo provider-private-error >&2; exit 7; }
printf resolved-jwks
`, { mode: 0o700 });
      if (scenario === 'denied') writeFileSync(join(dir, 'deny'), '');
      writeFileSync(join(dir, 'path/pnpm'), `#!/bin/sh
[ "$CLERK_JWKS_URL" = resolved-jwks ] || exit 10
[ "$VITE_API_BASE_URL" = http://localhost:8000 ] || exit 11
[ "$GROCERY_ENV_FILE" = /dev/null ] || exit 12
[ -z "$OP_SERVICE_ACCOUNT_TOKEN$OP_CONNECT_TOKEN$OP_SESSION_fixture$CLERK_SECRET_KEY$VITE_STALE$DATABASE_URL$STORE_TAB_SECRET" ] || exit 13
printf build-started
[ "${scenario}" != build-failure ]
`, { mode: 0o700 });
      const result = spawnSync(process.execPath, [join(dir, 'scripts/build-frontend.mjs')], {
        encoding: 'utf8',
        env: { ...process.env, PATH: `${dir}/path:${process.env.PATH}`,
          GROCERY_OP_TOKENS: `${dir}/etc/app.token ${dir}/etc/shared.token`, GROCERY_ETC: `${dir}/etc`,
          GROCERY_PATH: `${dir}/path:/usr/bin:/bin`, SCENARIO: scenario,
          OP_SERVICE_ACCOUNT_TOKEN: 'stale-export', OP_CONNECT_TOKEN: 'stale', OP_SESSION_fixture: 'stale',
          CLERK_SECRET_KEY: 'stale-secret', VITE_STALE: 'stale', DATABASE_URL: 'stale-database', STORE_TAB_SECRET: 'stale-store' },
      });
      const builds = ['success', 'shared-token', 'build-failure'].includes(scenario);
      assert.equal(result.stdout, builds ? 'build-started' : '', result.stderr);
      assert.equal(result.status === 0, ['success', 'shared-token'].includes(scenario), result.stderr);
      assert.doesNotMatch(result.stdout + result.stderr, /fixture-app|fixture-shared|provider-private-error|stale-secret/);
      if (!['empty-token', 'sudo-failure'].includes(scenario)) assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), 'service-attempt\n');
    } finally { rmSync(dir, { recursive: true }); }
  });
}
