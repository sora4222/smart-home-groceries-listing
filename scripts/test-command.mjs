/** Compact test output without losing the command's failure status or full log. */
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { stripVTControlCharacters } from 'node:util';
import { cancellation, runProcess } from './test-process.mjs';

const [label, cwd, command, ...args] = process.argv.slice(2);
const dir = mkdtempSync(join(tmpdir(), 'grocery-test-command-'));
const log = join(dir, 'output.log');
try {
  if (!label || !cwd || !command) throw new Error('Usage: test-command.mjs LABEL DIRECTORY COMMAND [ARGS...]');
  await runProcess(command, args, { cwd, log, signal: cancellation() });
  const output = stripVTControlCharacters(readFileSync(log, 'utf8'));
  if (/\b[1-9]\d* (skipped|ignored)\b/.test(output)) throw new Error(`${label} has unexpected skipped tests`);
  const summary = output.match(/^\s*(?:Tests\s+)?\d+ passed\b.*$/gm)?.at(-1)?.trim().replace(/^Tests\s+/, '');
  console.log(`${label}: ${summary ?? 'passed'}`);
  rmSync(dir, { recursive: true });
} catch (error) {
  console.error(error.message);
  try { console.error(readFileSync(log, 'utf8').split('\n').slice(-40).join('\n')); } catch {}
  console.error(`Test log: ${log}`);
  process.exitCode ||= 1;
}
