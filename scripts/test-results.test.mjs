/** Test verdicts depend on report artifacts, never application log wording. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

test('successful report survives skipped-row diagnostics; structured skips and missing reports fail', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-results-'));
  try {
    for (const mode of ['passed', 'skipped', 'missing', 'nonzero']) {
      const fixture = join(dir, 'fixture.mjs');
      writeFileSync(fixture, `import {writeFileSync} from 'node:fs'; console.log('1 skipped row\\nTests 1 passed'); const file=process.argv.find(a=>a.startsWith('--outputFile='))?.split('=').slice(1).join('='); if(${JSON.stringify(mode)}!=='missing' && file) writeFileSync(file,JSON.stringify({numPassedTests:${['passed', 'nonzero'].includes(mode) ? 1 : 0},numFailedTests:0,numPendingTests:${mode === 'skipped' ? 1 : 0},numTodoTests:0})); process.exit(${mode === 'nonzero' ? 7 : 0});`);
      for (const output of [[], ['--human']]) {
        const result = spawnSync(process.execPath, ['scripts/test-command.mjs', ...output, 'vitest', dir, process.execPath, fixture], { encoding: 'utf8' });
        assert.equal(result.status === 0, mode === 'passed', result.stderr);
        if (mode !== 'passed') assert.equal(result.stderr.includes('Test log:'), output.length === 0);
      }
    }
  } finally { rmSync(dir, { recursive: true }); }
});

test('framework adapters apply the same skip, failure, and empty-suite policy', async () => {
  const { readResults, verifyResults } = await import('./test-results.mjs');
  const dir = mkdtempSync(join(tmpdir(), 'grocery-adapters-'));
  try {
    const file = join(dir, 'report');
    for (const framework of ['vitest', 'pytest', 'playwright', 'libtest']) {
      for (const state of ['passed', 'failed', 'skipped', 'empty']) {
        const counts = { passed: +(state === 'passed'), failed: +(state === 'failed'), skipped: +(state === 'skipped') };
        const reports = {
          vitest: { numPassedTests: counts.passed, numFailedTests: counts.failed, numPendingTests: counts.skipped, numTodoTests: 0 },
          pytest: counts,
          playwright: { stats: { expected: counts.passed, unexpected: counts.failed, skipped: counts.skipped, flaky: 0 } },
        };
        writeFileSync(file, framework === 'libtest' ? ({ passed: 'ok fixture\n', failed: 'failed fixture\n', skipped: 'ignored fixture\n', empty: '' }[state]) : JSON.stringify(reports[framework]));
        const result = readResults(framework, file);
        assert.deepEqual(result, counts);
        if (state === 'passed') verifyResults(framework, result);
        else assert.throws(() => verifyResults(framework, result));
      }
    }
    writeFileSync(file, '{}');
    assert.throws(() => verifyResults('pytest', readResults('pytest', file)), /invalid result/);
  } finally { rmSync(dir, { recursive: true }); }
});

test('unit runners ignore household configuration and secret-provider credentials in both modes', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-unit-environment-'));
  try {
    const fixture = join(dir, 'fixture.mjs');
    writeFileSync(fixture, `import assert from 'node:assert/strict'; import {writeFileSync} from 'node:fs';
      assert.equal(process.env.GROCERY_ENV_FILE, '/dev/null');
      for (const key of ['DATABASE_URL', 'VITE_CLERK_PUBLISHABLE_KEY', 'OP_SERVICE_ACCOUNT_TOKEN']) assert.equal(process.env[key], undefined);
      const file=process.argv.find(a=>a.startsWith('--outputFile=')).slice('--outputFile='.length);
      writeFileSync(file, JSON.stringify({numPassedTests:1,numFailedTests:0,numPendingTests:0,numTodoTests:0}));`);
    for (const mode of [[], ['--human']]) {
      const result = spawnSync(process.execPath, ['scripts/test-command.mjs', ...mode, 'vitest', dir, process.execPath, fixture], {
        encoding: 'utf8', env: { ...process.env, GROCERY_ENV_FILE: '/missing/household.env', DATABASE_URL: 'fixture', VITE_CLERK_PUBLISHABLE_KEY: 'op://fixture/key', OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token' },
      });
      assert.equal(result.status, 0, result.stderr);
    }
  } finally { rmSync(dir, { recursive: true }); }
});
