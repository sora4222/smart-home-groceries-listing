/** Make's public commands compose once and preserve the existing lifecycle boundaries. */
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, cpSync, existsSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { testEnvironment } from './test-environment.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const dry = (...targets) => {
  const result = spawnSync('make', ['--no-print-directory', '-n', ...targets], { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
};

test('all runs format, lint, types, all tests once, and component builds', () => {
  const output = dry('all');
  for (const command of ['cargo fmt', 'ruff format', 'biome format --write', 'clippy', 'ruff check --fix', 'check --write', 'tsc --noEmit', 'backend-tests.sh', 'e2e-tests.mjs', 'desktop-tests.mjs']) {
    assert.ok(output.includes(command), command);
  }
  assert.equal(output.match(/backend-tests.sh/g)?.length, 1);
  for (const component of ['backend', 'frontend', 'alexa']) assert.ok(output.includes(`build-component.sh ${component}`));
  assert.ok(output.includes('deploy/build.sh --assemble'));
});

test('legacy commands delegate to canonical targets without duplicate recipes', () => {
  assert.equal(dry('stop-all'), dry('dev-stop'));
  assert.equal(dry('hm-install-server'), dry('install-server'));
  assert.equal(dry('hm-uninstall-server'), dry('uninstall-server'));
  assert.match(dry('hm-test-frontend'), /--human/);
  assert.equal(dry('build-desktop'), dry('create-application-desktop'));
});

test('compact command output hides successful chatter and preserves failures', () => {
  const command = join(root, 'scripts/command.mjs');
  const run = (...args) => spawnSync(process.execPath, [command, ...args], { encoding: 'utf8' });
  const success = run('fixture', '--', process.execPath, '-e', 'console.log("detail")');
  assert.equal(success.status, 0, success.stderr);
  assert.doesNotMatch(success.stdout, /detail/);
  const human = run('--human', 'fixture', '--', process.execPath, '-e', 'console.log("detail")');
  assert.equal(human.status, 0, human.stderr);
  assert.match(human.stdout, /detail/);
  const failure = run('fixture', '--', process.execPath, '-e', 'console.error("fixture failure"); process.exit(3)');
  assert.notEqual(failure.status, 0);
  assert.match(failure.stderr, /fixture failure/);
  const log = failure.stderr.match(/Command log: (.+)/)?.[1];
  assert.ok(log);
  rmSync(join(log, '..'), { recursive: true });
});

test('database reset refuses to act without explicit confirmation', () => {
  const result = spawnSync('make', ['reset-db'], { cwd: root, encoding: 'utf8', env: { ...process.env, GROCERY_ENV_FILE: '/does/not/exist' } });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /confirm=reset/);
  assert.doesNotMatch(result.stderr, /Cannot read|Cannot start docker/);
});

test('dependency installers require opt-in before downloading missing packages', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-install-guard-'));
  try {
    // The guard must execute before curl, apt, systemctl, or any installation.
    for (const name of ['curl', 'apt-get', 'systemctl']) writeFileSync(join(dir, name), '#!/bin/sh\necho unexpected-mutation >&2\nexit 99\n', { mode: 0o755 });
    for (const script of ['install-ollama.sh', 'install-cloudflared.sh']) {
      const result = spawnSync('/bin/bash', [join(root, 'deploy', script), 'fixture-model'], { encoding: 'utf8', env: { PATH: dir } });
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /ALLOW_DEPENDENCY_INSTALL=1/);
      assert.doesNotMatch(result.stderr, /unexpected-mutation/);
    }
  } finally { rmSync(dir, { recursive: true }); }
});

test('backend component build replaces stale output only on success', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-build-'));
  try {
    mkdirSync(join(dir, 'deploy'));
    mkdirSync(join(dir, 'bin'));
    mkdirSync(join(dir, 'build/backend'), { recursive: true });
    cpSync(join(root, 'deploy/build-component.sh'), join(dir, 'deploy/build-component.sh'));
    writeFileSync(join(dir, 'build/backend/stale'), 'old output');
    writeFileSync(join(dir, 'bin/cargo'), '#!/bin/sh\n[ -z "$FAIL_BUILD" ] || exit 42\nmkdir -p "$CARGO_TARGET_DIR/release"\nprintf fixture > "$CARGO_TARGET_DIR/release/grocery-backend"\n', { mode: 0o755 });
    const run = fail => spawnSync('bash', [join(dir, 'deploy/build-component.sh'), 'backend'], {
      encoding: 'utf8', env: { ...process.env, PATH: `${dir}/bin:${process.env.PATH}`, FAIL_BUILD: fail ? '1' : '', CARGO_TARGET_DIR: '/must/not/write/here' },
    });
    assert.notEqual(run(true).status, 0);
    assert.equal(readFileSync(join(dir, 'build/backend/stale'), 'utf8'), 'old output');
    const success = run(false);
    assert.equal(success.status, 0, success.stderr);
    assert.equal(existsSync(join(dir, 'build/backend/stale')), false);
    assert.equal(readFileSync(join(dir, 'build/backend/grocery-backend'), 'utf8'), 'fixture');
    assert.deepEqual(readdirSync(join(dir, 'build')), ['backend']);
  } finally { rmSync(dir, { recursive: true }); }
});

test('confirmed reset stops the owned backend and migrates only the local db', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-reset-'));
  try {
    writeFileSync(join(dir, '.env'), 'DATABASE_URL=postgres://fixture:fixture@remote.invalid/household\nPOSTGRES_PASSWORD=fixture\n');
    writeFileSync(join(dir, 'docker'), `#!${process.execPath}\nconst fs = require('node:fs'); fs.appendFileSync(process.env.RESET_LOG, JSON.stringify({ args: process.argv.slice(2), local: process.env.DATABASE_URL?.includes('@db:5432/') }) + '\\n');\n`, { mode: 0o755 });
    const result = spawnSync(process.execPath, [join(root, 'scripts/reset-db.mjs'), '--confirm'], {
      encoding: 'utf8', env: { ...testEnvironment(), PATH: `${dir}:${process.env.PATH}`, GROCERY_ENV_FILE: join(dir, '.env'), DATABASE_URL: 'postgres://fixture:fixture@remote.invalid/household', POSTGRES_PASSWORD: 'fixture', RESET_LOG: join(dir, 'calls') },
    });
    assert.equal(result.status, 0, result.stderr);
    const calls = readFileSync(join(dir, 'calls'), 'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(calls.length, 3);
    assert.deepEqual(calls[0].args.slice(-2), ['stop', 'backend']);
    assert.ok(calls[1].args.includes('exec'));
    assert.ok(calls[1].args.includes('db'));
    assert.deepEqual(calls[2].args.slice(-3), ['backend', 'grocery-backend', '--migrate-only']);
    assert.ok(calls.every(call => call.local));
    assert.ok(calls.every(call => !call.args.some(arg => arg.includes('postgres://'))));
  } finally { rmSync(dir, { recursive: true }); }
});

test('server assembly preserves the installed layout and removes stale bundle files', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-assembly-'));
  try {
    for (const name of ['deploy', 'scripts', 'backend/migrations', 'build/backend', 'build/alexa/venv', 'build/frontend/.output', 'build/production']) {
      mkdirSync(join(dir, name), { recursive: true });
    }
    cpSync(join(root, 'deploy/build.sh'), join(dir, 'deploy/build.sh'));
    cpSync(join(root, 'deploy/bin'), join(dir, 'deploy/bin'), { recursive: true });
    cpSync(join(root, 'deploy/systemd'), join(dir, 'deploy/systemd'), { recursive: true });
    for (const file of ['environment.mjs', 'op-auth.mjs', 'op-environment.mjs', 'service-env.mjs', 'service-components.mjs', 'systemd-env.mjs', 'database-setup-sql.mjs', 'triage-setup.mjs', 'migration-plan.mjs', 'run-with-env.mjs']) {
      cpSync(join(root, 'scripts', file), join(dir, 'scripts', file));
    }
    writeFileSync(join(dir, 'build/production/stale'), 'old bundle');
    const assemble = () => spawnSync('bash', [join(dir, 'deploy/build.sh'), '--assemble'], { encoding: 'utf8' });
    assert.notEqual(assemble().status, 0);
    assert.ok(existsSync(join(dir, 'build/production/stale')));
    writeFileSync(join(dir, 'build/backend/grocery-backend'), 'fixture binary');
    writeFileSync(join(dir, 'build/alexa/venv/fixture'), 'bridge');
    writeFileSync(join(dir, 'build/frontend/.output/fixture'), 'frontend');
    const result = assemble();
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(dir, 'build/production/stale')), false);
    for (const file of ['bin/grocery-backend', 'alexa-bridge/venv/fixture', 'frontend/.output/fixture', 'libexec/grocery-env', 'lib/service-env.mjs', 'systemd/grocery-backend.service', 'VERSION']) {
      assert.ok(existsSync(join(dir, 'build/production', file)), file);
    }
    assert.ok(!readdirSync(join(dir, 'build')).some(name => name.startsWith('.production.')));
  } finally { rmSync(dir, { recursive: true }); }
});
