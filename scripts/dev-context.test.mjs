/** Worktree identities isolate development resources and honour explicit settings. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { developmentContext } from './dev-context.mjs';

test('worktrees have stable independent identities and ports', () => {
  const a = mkdtempSync(join(tmpdir(), 'grocery-tree-a-'));
  const b = mkdtempSync(join(tmpdir(), 'grocery-tree-b-'));
  try {
    const first = developmentContext({}, a);
    assert.deepEqual(developmentContext({}, a), first);
    assert.notEqual(first.COMPOSE_PROJECT_NAME, developmentContext({}, b).COMPOSE_PROJECT_NAME);
    assert.notEqual(first.WEB_PORT, first.BACKEND_PORT);
    assert.deepEqual(developmentContext({ COMPOSE_PROJECT_NAME: 'chosen', WEB_PORT: '3001', BACKEND_PORT: '8001' }, a), {
      COMPOSE_PROJECT_NAME: 'chosen', WEB_PORT: '3001', BACKEND_PORT: '8001',
    });
    for (const value of ['abc', '80', '65536', '3000:3000']) {
      assert.throws(() => developmentContext({ WEB_PORT: value }, a), /Invalid WEB_PORT/);
    }
  } finally { rmSync(a, { recursive: true }); rmSync(b, { recursive: true }); }
});
