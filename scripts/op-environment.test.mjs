/** 1Password Developer Environment reads; every value here is a test fixture. */
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { loadEnvironment } from './environment.mjs';
import { readEnvironmentVariables } from './op-environment.mjs';

/** A fake `op run --environment ID --no-masking -- COMMAND` that adds two variables. */
const FAKE_OP = `#!/bin/sh
[ "$1" = run ] && [ "$2" = --environment ] && [ "$3" = envfixtureid ] && [ "$4" = --no-masking ] && [ "$5" = -- ] || exit 2
if [ -n "$OP_SERVICE_ACCOUNT_TOKEN" ]; then echo service; else echo desktop; fi >> "$AUTH_TRACE"
[ -z "$DENY_SERVICE" ] || [ -z "$OP_SERVICE_ACCOUNT_TOKEN" ] || { echo provider-private-error >&2; exit 7; }
shift 5
# Unlike this shell, the real CLI adds no shell variables of its own.
exec env -u PWD -u OLDPWD -u SHLVL -u _ VITE_API_BASE_URL='https://environment.example:8101' SERVER_API_BASE_URL=http://127.0.0.1:8000 "$@"
`;

function withFakeOp(run) {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-op-environment-'));
  try {
    writeFileSync(join(dir, 'op'), FAKE_OP, { mode: 0o700 });
    return run(dir, { PATH: `${dir}:/usr/bin:/bin`, AUTH_TRACE: join(dir, 'trace') });
  } finally { rmSync(dir, { recursive: true }); }
}

test('reads only the variables the Environment adds, with the service account first', () => withFakeOp((dir, base) => {
  const values = readEnvironmentVariables('envfixtureid', { ...base, OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token', UNRELATED: 'kept-out' });
  assert.deepEqual(values, { SERVER_API_BASE_URL: 'http://127.0.0.1:8000', VITE_API_BASE_URL: 'https://environment.example:8101' });
  assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), 'service\n');
}));

test('production service-only reads never fall back to the desktop app', () => withFakeOp((dir, base) => {
  const inherited = { ...base, OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token', GROCERY_OP_SERVICE_ONLY: '1', DENY_SERVICE: '1' };
  assert.throws(() => readEnvironmentVariables('envfixtureid', inherited), error => {
    assert.equal(error.message, 'Cannot read the 1Password Environment; check CLI access and the Environment ID');
    return true;
  });
  assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), 'service\n');
}));

test('a denied service account retries through the desktop app outside production', () => withFakeOp((dir, base) => {
  const values = readEnvironmentVariables('envfixtureid', { ...base, OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token', DENY_SERVICE: '1' });
  assert.equal(values.VITE_API_BASE_URL, 'https://environment.example:8101');
  assert.equal(readFileSync(join(dir, 'trace'), 'utf8'), 'service\ndesktop\n');
}));

test('Environment IDs that could be read as options are refused', () => {
  for (const id of ['', '--help', 'a b', 'id;rm']) {
    assert.throws(() => readEnvironmentVariables(id, {}), /Invalid 1Password Environment ID/);
  }
});

test('the Environment overrides dotenv, and exported values override both', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-layers-'));
  try {
    const file = join(dir, '.env');
    writeFileSync(file, 'VITE_API_BASE_URL=http://dotenv\nVITE_WS_URL=ws://dotenv\nVITE_EXPORTED=dotenv\n');
    const result = loadEnvironment({
      file, environmentId: 'envfixtureid', inherited: { VITE_EXPORTED: 'exported' },
      readEnvironment: id => { assert.equal(id, 'envfixtureid'); return { VITE_API_BASE_URL: 'https://environment', VITE_EXPORTED: 'environment' }; },
    });
    assert.deepEqual(result, { VITE_API_BASE_URL: 'https://environment', VITE_WS_URL: 'ws://dotenv', VITE_EXPORTED: 'exported' });
  } finally { rmSync(dir, { recursive: true }); }
});

test('without an Environment ID the Environment is never read', () => {
  const result = loadEnvironment({
    file: '/nonexistent/fixture.env', required: false, inherited: { VITE_A: 'a' },
    readEnvironment: () => { throw new Error('unexpected Environment read'); },
  });
  assert.deepEqual(result, { VITE_A: 'a' });
});
