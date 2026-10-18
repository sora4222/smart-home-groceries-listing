/** Production service configuration: per-service scope, token handling and output. */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { accepts, COMPONENTS, component } from './service-components.mjs';
import { parseArguments, resolveComponent } from './service-env.mjs';
import { formatSystemdEnvironment } from './systemd-env.mjs';

const FIXTURE = [
  'DATABASE_URL=postgres://grocery:pw@localhost:5432/grocery',
  'CLERK_SECRET_KEY=sk_test_fixture',
  'VITE_API_BASE_URL=http://localhost:8000',
  'ALEXA_SKILL_ID=amzn1.ask.skill.fixture',
  'ALEXA_BRIDGE_SECRET=bridge-fixture',
  'CLOUDFLARE_TUNNEL_TOKEN=tunnel-fixture',
  'BIND_ADDRESS=0.0.0.0:8000',
  'OP_SERVICE_ACCOUNT_TOKEN=never-forwarded',
].join('\n');

function withDir(run) {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-service-env-'));
  try { return run(dir); } finally { rmSync(dir, { recursive: true }); }
}

test('each service receives only its own keys', () => withDir(dir => {
  const file = join(dir, 'grocery.env');
  writeFileSync(file, FIXTURE);
  const bridge = resolveComponent({ name: 'alexa-bridge', file, inherited: {} });
  assert.deepEqual(Object.keys(bridge).sort(), ['ALEXA_BRIDGE_SECRET', 'ALEXA_SKILL_ID']);
  const frontend = resolveComponent({ name: 'frontend', file, inherited: {} });
  assert.deepEqual(Object.keys(frontend).sort(), ['CLERK_SECRET_KEY', 'VITE_API_BASE_URL']);
  const backend = resolveComponent({ name: 'backend', file, inherited: {} });
  assert.ok(backend.DATABASE_URL && backend.ALEXA_BRIDGE_SECRET);
  assert.equal(backend.CLERK_SECRET_KEY, undefined);
}));

test('listening addresses and 1Password credentials are never passed through', () => {
  for (const name of Object.keys(COMPONENTS)) {
    const spec = component(name);
    for (const key of ['BIND_ADDRESS', 'ACCESS_LOG_DIR', 'OP_SERVICE_ACCOUNT_TOKEN', 'OP_SESSION_x', 'GROCERY_ENV_FILE']) {
      assert.equal(accepts(spec, key), false, `${name} accepted ${key}`);
    }
  }
});

test('the tunnel token is renamed to what cloudflared reads', () => withDir(dir => {
  const file = join(dir, 'grocery.env');
  writeFileSync(file, FIXTURE);
  assert.deepEqual(resolveComponent({ name: 'tunnel', file, inherited: {} }), { TUNNEL_TOKEN: 'tunnel-fixture' });
}));

test('references resolve with the token file, and only the selected keys are resolved', () => withDir(dir => {
  const file = join(dir, 'grocery.env');
  const tokenFile = join(dir, 'token');
  writeFileSync(file, 'ALEXA_SKILL_ID=op://Vault/item/skill\nDATABASE_URL=op://Vault/item/never-read\n');
  writeFileSync(tokenFile, 'fixture-token\n');
  const seen = [];
  const values = resolveComponent({
    name: 'alexa-bridge', file, tokenFile, inherited: {},
    resolve: (reference, env) => { seen.push([reference, env.OP_SERVICE_ACCOUNT_TOKEN]); return 'resolved'; },
  });
  assert.deepEqual(values, { ALEXA_SKILL_ID: 'resolved' });
  assert.deepEqual(seen, [['op://Vault/item/skill', 'fixture-token']]);
}));

test('unknown components and formats are refused', () => {
  assert.throws(() => component('db'), /Unknown component/);
  assert.throws(() => parseArguments(['--format', 'yaml', 'backend']), /Unknown format/);
  assert.throws(() => parseArguments([]), /Usage/);
});

test('systemd output escapes special characters and refuses line breaks', () => {
  assert.equal(formatSystemdEnvironment({ A: 'a"b\\c$d`e #f' }), 'A="a\\"b\\\\c\\$d\\`e #f"\n');
  assert.throws(() => formatSystemdEnvironment({ SECRET: 'one\ntwo' }), /SECRET contains a line break/);
  assert.equal(formatSystemdEnvironment({}), '');
});

test('the command reads configuration from standard input and ignores exported values', () => {
  const result = spawnSync(process.execPath, ['scripts/service-env.mjs', '--format', 'json', 'tunnel'], {
    input: 'CLOUDFLARE_TUNNEL_TOKEN=from-stdin\n',
    env: { PATH: process.env.PATH, CLOUDFLARE_TUNNEL_TOKEN: 'exported', OP_SERVICE_ACCOUNT_TOKEN: 'x' },
    encoding: 'utf8',
  });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), { TUNNEL_TOKEN: 'from-stdin' });
});

test('the backend component matches the keys the Rust backend loads, minus unit-owned ones', () => {
  const source = readFileSync(new URL('../backend/src/environment/mod.rs', import.meta.url), 'utf8');
  const block = source.slice(source.indexOf('const KEYS'), source.indexOf('];', source.indexOf('const KEYS')));
  const rust = [...block.matchAll(/"([A-Z0-9_]+)"/g)].map(match => match[1]);
  const unitOwned = ['BIND_ADDRESS', 'ACCESS_LOG_DIR'];
  assert.deepEqual([...COMPONENTS.backend.keys].sort(), rust.filter(key => !unitOwned.includes(key)).sort());
});
