/** Owned test subprocesses, with process-group cleanup and preserved failures. */
import { spawn } from 'node:child_process';
import { openSync, closeSync } from 'node:fs';
import { setTimeout as delay } from 'node:timers/promises';

/** Start a process in its own group; stop only that group on cancellation. */
export function startProcess(command, args, { cwd, env, log, stdout, signal } = {}) {
  signal?.throwIfAborted();
  const fd = log ? openSync(log, 'a', 0o600) : null;
  const out = stdout ? openSync(stdout, 'w', 0o600) : fd;
  const child = spawn(command, args, {
    cwd, env, detached: process.platform !== 'win32',
    stdio: ['ignore', out ?? 'inherit', fd ?? 'inherit'],
  });
  if (fd !== null) closeSync(fd);
  if (stdout) closeSync(out);
  let ended = false;
  const completed = new Promise((resolve, reject) => {
    child.once('error', () => { ended = true; reject(new Error(`Cannot start ${command}`)); });
    child.once('exit', (code, exitSignal) => {
      ended = true;
      if (code === 0) resolve();
      else reject(new Error(`${command} failed (${exitSignal ?? code})`));
    });
  });
  // A server may exit while its owner is awaiting readiness or another child.
  completed.catch(() => {});
  const kill = sig => {
    if (!child.pid) return;
    try {
      if (process.platform === 'win32') child.kill(sig);
      else process.kill(-child.pid, sig);
    } catch (error) { if (error.code !== 'ESRCH') throw error; }
  };
  const stop = async () => {
    kill('SIGTERM');
    await Promise.race([completed.catch(() => {}), delay(3000, undefined, { ref: false })]);
    kill('SIGKILL');
  };
  const abort = () => { void stop(); };
  signal?.addEventListener('abort', abort, { once: true });
  completed.finally(() => signal?.removeEventListener('abort', abort)).catch(() => {});
  return {
    completed, stop,
    assertRunning() { if (ended) throw new Error(`${command} exited before tests completed`); },
  };
}

/** Run a command to completion while retaining its error and cleaning descendants. */
export async function runProcess(command, args, options) {
  const child = startProcess(command, args, options);
  try { await child.completed; }
  finally { await child.stop(); }
}

/** Convert shell interrupts into cancellation so async resource cleanup can run. */
export function cancellation() {
  const controller = new AbortController();
  for (const [name, code] of [['SIGINT', 130], ['SIGTERM', 143]]) {
    process.once(name, () => { process.exitCode = code; controller.abort(); });
  }
  return controller.signal;
}

/** Fail immediately when any owned server exits while work is still running. */
export async function withServers(servers, work) {
  const controller = new AbortController();
  const running = work(controller.signal);
  try {
    return await Promise.race([
      running,
      ...servers.map(server => server.completed.then(() => {
        throw new Error('Test server exited before tests completed');
      })),
    ]);
  } finally {
    controller.abort();
    await running.catch(() => {});
  }
}
