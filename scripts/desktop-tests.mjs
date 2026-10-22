/** Native unit tests use the repository's structured Cargo verdict policy. */
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { cargoResults } from './cargo-results.mjs';
import { desktopEnvironment } from './desktop-context.mjs';
import { cancellation } from './test-process.mjs';
import { verifyResults } from './test-results.mjs';

const dir = mkdtempSync(join(tmpdir(), 'grocery-desktop-tests-'));
const log = join(dir, 'cargo.log');
try {
  const result = await cargoResults([], {
    dir, cwd: new URL('../desktop/src-tauri/', import.meta.url), log,
    env: desktopEnvironment(process.env), signal: cancellation(),
  });
  verifyResults('desktop', result);
  rmSync(dir, { recursive: true });
} catch (error) {
  console.error(error.message);
  try { console.error(readFileSync(log, 'utf8').split('\n').slice(-30).join('\n')); } catch {}
  console.error(`Desktop test log: ${log}`);
  process.exitCode ||= 1;
}
