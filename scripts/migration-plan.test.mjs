/** `make hm-update-server` migration checks: pending files, destructive statements, downgrades. */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { destructiveStatements, migrationPlan, migrationVersion } from './migration-plan.mjs';
import { lookupUser } from './run-with-env.mjs';

test('versions follow sqlx: the numeric prefix, so 0002 and 20261002120000 both parse', () => {
  assert.equal(migrationVersion('0002_notes.sql'), 2n);
  assert.equal(migrationVersion('20261002120000_purchase_history.sql'), 20261002120000n);
  assert.equal(migrationVersion('README.md'), undefined);
});

test('destructive statements are found, but not in comments, strings or index drops', () => {
  assert.deepEqual(destructiveStatements('ALTER TABLE a DROP COLUMN b;'), ['DROP COLUMN']);
  assert.deepEqual(destructiveStatements('alter table a alter column b type text;'), ['column type change']);
  assert.deepEqual(destructiveStatements('DELETE FROM items; TRUNCATE t;'), ['TRUNCATE', 'DELETE FROM']);
  assert.deepEqual(destructiveStatements("-- DROP TABLE x\n/* TRUNCATE y */ INSERT INTO n VALUES ('DROP TABLE z');"), []);
  assert.deepEqual(destructiveStatements('DROP INDEX IF EXISTS i; ALTER TABLE a DROP CONSTRAINT c;'), []);
  assert.deepEqual(destructiveStatements('DO $$ BEGIN RAISE NOTICE \'DELETE FROM\'; END $$;'), []);
});

test('only unapplied files are pending, in version order', () => {
  const files = ['0003_c.sql', '0001_a.sql', '0002_b.sql'];
  const plan = migrationPlan(files, ['1'], () => 'CREATE TABLE t (id int);');
  assert.deepEqual(plan.pending.map(({ file }) => file), ['0002_b.sql', '0003_c.sql']);
  assert.deepEqual(plan.ahead, []);
});

test('a database with migrations the checkout lacks is reported as ahead', () => {
  assert.deepEqual(migrationPlan(['0001_a.sql'], ['1', '2'], () => '').ahead, ['2']);
});

test('none of the repository migrations is destructive (they are all already applied in production)', () => {
  const dir = new URL('../backend/migrations/', import.meta.url);
  const plan = migrationPlan(readdirSync(dir), [], file => readFileSync(new URL(file, dir), 'utf8'));
  assert.ok(plan.pending.length > 0);
  for (const { file, destructive } of plan.pending) assert.deepEqual(destructive, [], file);
});

test('the command exits 3 for a destructive pending migration and 4 for a downgrade', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-migrations-'));
  try {
    writeFileSync(join(dir, '0001_a.sql'), 'CREATE TABLE a (id int);');
    writeFileSync(join(dir, '0002_b.sql'), 'DROP TABLE a;');
    const run = input => spawnSync(process.execPath, ['scripts/migration-plan.mjs', dir], { input, encoding: 'utf8' });
    assert.equal(run('1\n2\n').status, 0);
    const destructive = run('1\n');
    assert.equal(destructive.status, 3);
    assert.match(destructive.stdout, /destructive 0002_b.sql: DROP TABLE/);
    assert.equal(run('').status, 3);
    assert.equal(run('1\n2\n3\n').status, 4);
  } finally { rmSync(dir, { recursive: true }); }
});

test('service users are looked up from passwd', () => {
  assert.deepEqual(lookupUser('nobody', 'root:x:0:0::/root:/bin/sh\nnobody:x:65534:65534::/:/usr/sbin/nologin\n'), { uid: 65534, gid: 65534 });
  assert.throws(() => lookupUser('ghost', ''), /No such user/);
});
