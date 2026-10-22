/** Run stable Rust test harnesses with dedicated per-test result records. */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { runProcess } from './test-process.mjs';
import { readResults } from './test-results.mjs';

/** Discover executables via Cargo JSON, preserving Cargo selectors and harness args. */
export async function cargoResults(args, { dir, ...options }) {
  const separator = args.indexOf('--');
  const selection = separator < 0 ? args : args.slice(0, separator);
  const harness = separator < 0 ? [] : args.slice(separator + 1);
  const artifacts = join(dir, 'artifacts.jsonl');
  await runProcess('cargo', ['test', '--locked', '--no-run', '--message-format=json', ...selection], { ...options, stdout: artifacts });
  const binaries = [...new Set(readFileSync(artifacts, 'utf8').split('\n').filter(Boolean)
    .map(line => JSON.parse(line)).filter(message => message.reason === 'compiler-artifact' && message.profile.test && message.executable)
    .map(message => message.executable))];
  if (!binaries.length) throw new Error('Cargo produced no test executables');
  const total = { passed: 0, failed: 0, skipped: 0 };
  for (const [index, binary] of binaries.entries()) {
    const file = join(dir, `libtest-${index}.results`);
    await runProcess(binary, [...harness, '--logfile', file], options);
    const result = readResults('libtest', file);
    for (const key of Object.keys(total)) total[key] += result[key];
  }
  // Cargo's default test command also runs documentation tests. Target selection
  // deliberately limits this phase in the same way as cargo test --test/--lib.
  if (!selection.length) {
    const file = join(dir, 'doctest.results');
    await runProcess('cargo', ['test', '--locked', '--doc', '--', ...harness, '--logfile', file], options);
    const result = readResults('libtest', file);
    for (const key of Object.keys(total)) total[key] += result[key];
  }
  return total;
}
