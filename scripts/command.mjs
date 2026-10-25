/** Keep successful command output compact; retain private diagnostics on failure. */
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { cancellation, runProcess } from './test-process.mjs';

const args = process.argv.slice(2);
const human = args[0] === '--human';
if (human) args.shift();
const [label, separator, command, ...commandArgs] = args;
const dir = mkdtempSync(join(tmpdir(), 'grocery-command-'));
const log = join(dir, 'output.log');
try {
  if (!label || separator !== '--' || !command) throw new Error('Usage: command.mjs [--human] LABEL -- COMMAND [ARGS...]');
  await runProcess(command, commandArgs, { log: human ? undefined : log, signal: cancellation() });
  rmSync(dir, { recursive: true });
  if (!human) console.log(`${label}: passed`);
} catch (error) {
  console.error(error.message);
  if (!human) {
    try { console.error(readFileSync(log, 'utf8').split('\n').slice(-40).join('\n')); } catch {}
    console.error(`Command log: ${log}`);
  } else rmSync(dir, { recursive: true });
  process.exitCode ||= 1;
}
