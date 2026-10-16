/** Lifecycle configuration must select the same stack without unlocking secrets. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

test('up and down share selected Compose identity; down needs no provider and preserves failures', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-lifecycle-'));
  try {
    const file = join(dir, 'selected.env');
    writeFileSync(file, 'COMPOSE_PROJECT_NAME=grocery-selected\nDATABASE_URL=postgres://fixture@db/grocery\nCLERK_SECRET_KEY=op://unavailable/secret\n');
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nprintf fixture-secret\n', { mode: 0o700 });
    writeFileSync(join(dir, 'docker'), '#!/bin/sh\nprintf "%s|%s\\n" "$COMPOSE_PROJECT_NAME" "$*"\nexit "${DOCKER_EXIT:-0}"\n', { mode: 0o700 });
    const env = { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: file };
    const up = spawnSync('make', ['up'], { env, encoding: 'utf8' });
    assert.equal(up.status, 0, up.stderr);
    assert.match(up.stdout, /^grocery-selected\|/);
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nexit 99\n', { mode: 0o700 });
    const down = spawnSync('make', ['down'], { env, encoding: 'utf8' });
    assert.equal(down.status, 0, down.stderr);
    assert.match(down.stdout, /^grocery-selected\|/);
    assert.ok(up.stdout.includes(`--env-file ${file}`));
    assert.ok(down.stdout.includes(`--env-file ${file}`));
    assert.notEqual(spawnSync('make', ['down'], { env: { ...env, DOCKER_EXIT: '7' } }).status, 0);
  } finally { rmSync(dir, { recursive: true }); }
});
