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
    const up = spawnSync('node', ['scripts/compose.mjs', 'up', '-d'], { env, encoding: 'utf8' });
    assert.equal(up.status, 0, up.stderr);
    assert.match(up.stdout, /^grocery-selected\|/);
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nexit 99\n', { mode: 0o700 });
    const down = spawnSync('node', ['scripts/compose.mjs', 'down'], { env, encoding: 'utf8' });
    assert.equal(down.status, 0, down.stderr);
    assert.match(down.stdout, /^grocery-selected\|/);
    assert.ok(up.stdout.includes(`--env-file ${file}`));
    assert.ok(down.stdout.includes(`--env-file ${file}`));
    assert.notEqual(spawnSync('node', ['scripts/compose.mjs', 'down'], { env: { ...env, DOCKER_EXIT: '7' } }).status, 0);
  } finally { rmSync(dir, { recursive: true }); }
});


test('development stop is repeatable without resolving application references', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-dev-stop-'));
  try {
    const file = join(dir, 'selected.env');
    writeFileSync(file, 'DATABASE_URL=op://unavailable/db\nCOMPOSE_PROJECT_NAME=isolated-dev\nWEB_PORT=44001\nBACKEND_PORT=44002\n');
    writeFileSync(join(dir, 'op'), '#!/bin/sh\nexit 99\n', { mode: 0o700 });
    writeFileSync(join(dir, 'docker'), '#!/bin/sh\nprintf "%s|%s|%s|%s\\n" "$COMPOSE_PROJECT_NAME" "$WEB_PORT" "$BACKEND_PORT" "$*"\n', { mode: 0o700 });
    const env = { PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: file };
    for (let attempt = 0; attempt < 2; attempt++) {
      const stopped = spawnSync('make', ['stop-all'], { env, encoding: 'utf8' });
      assert.equal(stopped.status, 0, stopped.stderr);
      assert.match(stopped.stdout, /^isolated-dev\|44001\|44002\|/);
      assert.match(stopped.stdout, /docker-compose.dev.yml.*down/);
    }
  } finally { rmSync(dir, { recursive: true }); }
});
