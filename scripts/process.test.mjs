/** Exercise launcher cancellation through a real child and grandchild. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

test('launcher termination stops descendants and retains signal failure', { timeout: 10000 }, async () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-process-'));
  const file = join(dir, 'pid');
  writeFileSync(join(dir, '.env'), '');
  const program = `const {spawn}=require('node:child_process'); const c=spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'}); require('node:fs').writeFileSync(${JSON.stringify(file)},String(c.pid)); process.on('SIGTERM',()=>process.exit(0)); setInterval(()=>{},1000);`;
  const launcher = spawn(process.execPath, ['scripts/run-env.mjs', process.execPath, '-e', program], {
    env: { PATH: process.env.PATH, GROCERY_ENV_FILE: join(dir, '.env') }, stdio: 'ignore',
  });
  let pid;
  try {
    for (let n = 0; n < 100 && !existsSync(file); n++) await delay(20);
    pid = Number(readFileSync(file, 'utf8'));
    const exit = once(launcher, 'exit');
    launcher.kill('SIGTERM');
    const [code] = await exit;
    let alive = true;
    for (let n = 0; n < 100 && alive; n++) {
      try { process.kill(pid, 0); await delay(20); } catch { alive = false; }
    }
    assert.equal(alive, false, 'grandchild survives launcher termination');
    assert.equal(code, 143);
  } finally {
    launcher.kill('SIGKILL');
    if (pid) { try { process.kill(pid, 'SIGKILL'); } catch {} }
    rmSync(dir, { recursive: true });
  }
});

test('server exit cancels and awaits owned work', async () => {
  const { startProcess, withServers } = await import('./test-process.mjs');
  const server = startProcess(process.execPath, ['-e', 'process.exit(0)']);
  let cleaned = false;
  try {
    await assert.rejects(withServers([server], async signal => {
      try { await delay(10000, undefined, { signal }); }
      finally { cleaned = true; }
    }), /server exited/);
    assert.equal(cleaned, true);
  } finally { await server.stop(); }
});
