/** Run the complete watched development stack and remove only its containers. */
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { runCompose } from './compose.mjs';
import { DEVELOPMENT_FILES, loadDevelopmentContext } from './dev-context.mjs';
import { cancellation } from './test-process.mjs';

/** This worktree's project and ports; when they are invalid nothing starts, so exit. */
function contextOrExit() {
  try { return loadDevelopmentContext(); }
  catch (error) { console.error(error.message); process.exit(1); }
}

const context = contextOrExit();
const signal = cancellation();
const human = process.argv.includes('--human');
const dir = mkdtempSync(join(tmpdir(), 'grocery-dev-'));
const run = ([action, ...args], stage, cancellable = true) => runCompose({
  action, args, files: DEVELOPMENT_FILES, overrides: context,
  signal: cancellable ? signal : undefined, log: human ? undefined : join(dir, `${stage}.log`),
});
try {
  console.log('dev: building and waiting for the local stack');
  await run(['up', '--build', '--wait'], 'startup');
  console.log(`dev: http://localhost:${context.WEB_PORT} (API http://localhost:${context.BACKEND_PORT})`);
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
