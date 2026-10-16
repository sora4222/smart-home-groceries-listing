/** The test database belongs to one run and must survive neither failure nor success. */
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { withTestDatabase } from './test-database.mjs';
import { testEnvironment } from './test-environment.mjs';
import { runProcess, startProcess } from './test-process.mjs';

for (const fails of [false, true]) {
  test(`temporary database is isolated and cleaned up after ${fails ? 'failure' : 'success'}`, async () => {
    const events = [];
    const failure = new Error('test failure');
    const start = async () => {
      events.push('ready');
      return { getConnectionUri: () => 'postgres://fixture@localhost:49152/postgres', stop: async () => events.push('stop') };
    };
    const promise = withTestDatabase(async url => {
      events.push('test');
      assert.equal(url, 'postgres://fixture@localhost:49152/postgres');
      if (fails) throw failure;
      return 7;
    }, { start });
    if (fails) await assert.rejects(promise, error => error === failure);
    else assert.equal(await promise, 7);
    assert.deepEqual(events, ['ready', 'test', 'stop']);
  });
}

test('an unavailable Docker database stops before running tests', async () => {
  const failure = new Error('Docker unavailable');
  await assert.rejects(withTestDatabase(() => assert.fail('tests must not start'), {
    start: async () => { throw failure; },
  }), error => error === failure);
});

test('test servers retain toolchains but cannot inherit application or provider credentials', () => {
  assert.deepEqual(testEnvironment({ PATH: '/tools', CARGO_HOME: '/cache', DATABASE_URL: 'private',
    OP_SERVICE_ACCOUNT_TOKEN: 'private', CLERK_SECRET_KEY: 'private', OPENAI_API_KEY: 'private',
    VITE_CLERK_PUBLISHABLE_KEY: 'private', E2E_API_BASE_URL: 'https://household.invalid',
  }), { PATH: '/tools', CARGO_HOME: '/cache' });
});

test('failed test subprocesses fail their owner', async () => {
  await assert.rejects(runProcess(process.execPath, ['-e', 'process.exit(7)']), /failed \(7\)/);
});

test('cancellation terminates the owned subprocess', async () => {
  const controller = new AbortController();
  const child = startProcess(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { signal: controller.signal });
  controller.abort();
  await assert.rejects(child.completed, /SIGTERM/);
  await child.stop();
});

for (const [code, output, succeeds] of [[0, '1 passed', true], [7, 'fixture failure', false], [0, '1 skipped', false]]) {
  test(`compact output preserves failure and skip results: ${code}, ${output}`, () => {
    const result = spawnSync(process.execPath, ['scripts/test-command.mjs', 'fixture', '.', process.execPath,
      '-e', `console.log(${JSON.stringify(output)}); process.exit(${code});`], { encoding: 'utf8' });
    assert.equal(result.status === 0, succeeds, result.stderr);
    if (succeeds) assert.equal(result.stdout.trim(), 'fixture: 1 passed');
    else assert.match(result.stderr, /Test log:/);
  });
}
