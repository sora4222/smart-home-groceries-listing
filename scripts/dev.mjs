/** Run the complete watched development stack and remove only its containers. */
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cancellation, runProcess } from './test-process.mjs';

const signal = cancellation();
const human = process.argv.includes('--human');
const dir = mkdtempSync(join(tmpdir(), 'grocery-dev-'));
const runner = fileURLToPath(new URL('./compose.mjs', import.meta.url));
const run = (args, stage, cancellable = true) => runProcess(process.execPath, [runner, '--development', ...args], {
  signal: cancellable ? signal : undefined,
  log: human ? undefined : join(dir, `${stage}.log`),
});
try {
  console.log('dev: building and waiting for the local stack');
  await run(['up', '--build', '--wait'], 'startup');
  if (!human) {
    const address = readFileSync(join(dir, 'startup.log'), 'utf8').split('\n').find(line => line.startsWith('dev: http://'));
    if (address) console.log(address);
  }
  console.log('dev: stack ready; watching sources. Ctrl-C stops this worktree.');
  await run(['up', '--watch', '--abort-on-container-exit'], 'watch');
  throw new Error('Development services exited while watch was active');
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
  if (!signal.aborted) {
    try { await run(['logs', '--no-color', '--tail', '30'], 'services', false); } catch {}
  }
} finally {
  try { await run(['down'], 'shutdown', false); }
  catch (error) { console.error(error.message); process.exitCode ||= 1; }
  if (process.exitCode && !human) {
    for (const stage of ['startup', 'watch', 'services', 'shutdown']) {
      try { console.error(readFileSync(join(dir, `${stage}.log`), 'utf8').split('\n').slice(-30).join('\n')); } catch {}
    }
  }
  console.log(`dev: logs ${dir}`);
}
