/** Shared configuration-source fixtures consumed by JavaScript, Python, and Rust. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { loadEnvironment } from './environment.mjs';

for (const fixture of JSON.parse(readFileSync(new URL('./environment-contract.json', import.meta.url)))) {
  test(fixture.name, () => {
    const dir = mkdtempSync(join(tmpdir(), 'grocery-contract-'));
    try {
      const path = join(dir, '.env');
      if (fixture.contents !== null) writeFileSync(path, fixture.contents);
      const file = fixture.source === 'empty' ? '' : fixture.source === 'directory' ? dir : path;
      const run = () => loadEnvironment({ file, required: fixture.source !== 'default', inherited: fixture.inherited });
      if (fixture.error) assert.throws(run, /Cannot read environment file/);
      else assert.equal(run().ALEXA_SKILL_ID, fixture.expected);
    } finally { rmSync(dir, { recursive: true }); }
  });
}
