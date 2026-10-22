/** Run SQLx tests against one disposable server without loading application secrets. */
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { withTestDatabase } from './test-database.mjs';
import { cargoResults } from './cargo-results.mjs';
import { verifyResults } from './test-results.mjs';
import { cancellation } from './test-process.mjs';
import { testEnvironment } from './test-environment.mjs';

const signal = cancellation();
const dir = mkdtempSync(join(tmpdir(), 'grocery-backend-tests-'));
const log = join(dir, 'cargo.log');
const args = process.argv.slice(2);
const compact = args[0] === '--compact';
if (compact) args.shift();
try {
  await withTestDatabase(async url => {
    const result = await cargoResults(args, {
      dir, cwd: new URL('../backend/', import.meta.url),
      env: { ...testEnvironment(), DATABASE_URL: url }, log: compact ? log : undefined, signal,
    });
    verifyResults('backend', result);
  });
  rmSync(dir, { recursive: true });
} catch (error) {
  console.error(error.message);
  if (compact) {
    try { console.error(readFileSync(log, 'utf8').split('\n').slice(-40).join('\n')); } catch {}
    console.error(`Backend test log: ${log}`);
  }
  process.exitCode ||= 1;
}
