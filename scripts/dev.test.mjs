/** Development startup owns a complete stack and always attempts safe cleanup. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

for (const fail of ['', '--wait', '--watch', 'down']) {
  test(`dev cleans up and preserves failures: ${fail || 'unexpected clean exit'}`, () => {
    const dir = mkdtempSync(join(tmpdir(), 'grocery-dev-test-'));
    try {
      const file = join(dir, 'selected.env');
      const calls = join(dir, 'calls');
      writeFileSync(file, 'DATABASE_URL=postgres://fixture@db/grocery\n');
      writeFileSync(join(dir, 'docker'), `#!/bin/sh\nprintf '%s\\n' "$*" >> "$CALLS"\ncase "$*" in *"$FAIL"*) [ -z "$FAIL" ] || exit 7;; esac\n`, { mode: 0o700 });
      const run = spawnSync('node', ['scripts/dev.mjs'], {
        env: { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: file, CALLS: calls, FAIL: fail }, encoding: 'utf8',
      });
      assert.notEqual(run.status, 0, run.stderr);
      const args = readFileSync(calls, 'utf8');
      assert.match(args, /up --build --wait/);
      assert.match(args, /down/);
      if (fail !== '--wait') assert.match(args, /up --watch --abort-on-container-exit/);
      assert.match(args, /docker-compose.dev.yml/);
      assert.ok(!args.includes('--volumes'));
    } finally { rmSync(dir, { recursive: true }); }
  });
}

test('SIGINT cleans up the watched stack and preserves exit 130', async () => {
  const { spawn } = await import('node:child_process');
  const { setTimeout: delay } = await import('node:timers/promises');
  const dir = mkdtempSync(join(tmpdir(), 'grocery-dev-signal-'));
  let child;
  try {
    const calls = join(dir, 'calls');
    const file = join(dir, 'selected.env');
    writeFileSync(file, 'DATABASE_URL=postgres://fixture@db/grocery\n');
    writeFileSync(join(dir, 'docker'), '#!/bin/sh\nprintf "%s\\n" "$*" >> "$CALLS"\ncase "$*" in *--watch*) exec sleep 30;; esac\n', { mode: 0o700 });
    child = spawn('node', ['scripts/dev.mjs'], {
      env: { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: file, CALLS: calls }, stdio: 'ignore',
    });
    const exited = new Promise(resolve => child.once('exit', code => resolve(code)));
    const deadline = Date.now() + 10000;
    while (!readFileSync(calls, { encoding: 'utf8', flag: 'a+' }).includes('--watch')) {
      assert.ok(Date.now() < deadline, 'watch must start');
      await delay(25);
    }
    child.kill('SIGINT');
    assert.equal(await exited, 130);
    assert.match(readFileSync(calls, 'utf8'), /down/);
  } finally { child?.kill('SIGKILL'); rmSync(dir, { recursive: true }); }
});
