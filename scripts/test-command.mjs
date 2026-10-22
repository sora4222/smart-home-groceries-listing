/** Compact test output without losing the command's failure status or full log. */
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readResults, verifyResults } from './test-results.mjs';
import { cancellation, runProcess } from './test-process.mjs';
import { testEnvironment } from './test-environment.mjs';

const input = process.argv.slice(2);
const human = input[0] === '--human';
if (human) input.shift();
const [label, cwd, command, ...args] = input;
const dir = mkdtempSync(join(tmpdir(), 'grocery-test-command-'));
const log = join(dir, 'output.log');
try {
  if (!label || !cwd || !command) throw new Error('Usage: test-command.mjs [--human] LABEL DIRECTORY COMMAND [ARGS...]');
  const report = join(dir, 'results.json');
  const env = { ...testEnvironment(), GROCERY_ENV_FILE: '/dev/null' };
  if (label === 'vitest') args.push(...(human ? ['--reporter=default'] : []), '--reporter=json', `--outputFile=${report}`);
  else if (label === 'pytest') {
    args.push('-p', 'grocery_pytest_report');
    env.GROCERY_TEST_REPORT = report;
    env.PYTHONPATH = [fileURLToPath(new URL('.', import.meta.url)), env.PYTHONPATH].filter(Boolean).join(':');
  } else throw new Error('Expected vitest or pytest adapter');
  await runProcess(command, args, { cwd, env, log: human ? undefined : log, signal: cancellation() });
  verifyResults(label, readResults(label, report));
  rmSync(dir, { recursive: true });
} catch (error) {
  console.error(error.message);
  if (!human) {
    try { console.error(readFileSync(log, 'utf8').split('\n').slice(-40).join('\n')); } catch {}
    console.error(`Test log: ${log}`);
  }
  process.exitCode ||= 1;
}
