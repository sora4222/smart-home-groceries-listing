/** `make hm-install-server` helpers: PostgreSQL role setup and the triage model plan. */
import assert from 'node:assert/strict';
import { createHash, createHmac, pbkdf2Sync } from 'node:crypto';
import { test } from 'node:test';
import { databaseTarget, scramVerifier, setupSql } from './database-setup-sql.mjs';
import { triagePlan } from './triage-setup.mjs';

test('a loopback URL is set up locally, with its percent-encoded parts decoded', () => {
  assert.deepEqual(databaseTarget('postgres://grocery:p%40ss@localhost:5432/grocery'), {
    local: true, user: 'grocery', password: 'p@ss', database: 'grocery',
  });
  assert.equal(databaseTarget('postgres://u:p@10.0.0.5/grocery').local, false);
});

test('Docker Compose hostnames are refused without echoing the password', () => {
  for (const host of ['db', 'host.docker.internal']) {
    assert.throws(() => databaseTarget(`postgres://grocery:topsecret@${host}:5432/grocery`), error => {
      assert.match(error.message, /Docker Compose host/);
      assert.doesNotMatch(error.message, /topsecret/);
      return true;
    });
  }
});

test('the SQL never contains the plain password and quotes identifiers', () => {
  const sql = setupSql({ user: 'gro"cery', password: "it's secret", database: "my'db" }, () => 'VERIFIER');
  assert.doesNotMatch(sql, /secret/);
  assert.match(sql, /ALTER ROLE "gro""cery" WITH LOGIN PASSWORD 'VERIFIER';/);
  assert.match(sql, /'my''db'/);
  assert.throws(() => setupSql({ user: 'postgres', password: 'x', database: 'd' }), /not postgres/);
});

test('the SCRAM verifier matches RFC 7677 key derivation', () => {
  const salt = Buffer.alloc(16, 7);
  const [head, keys] = scramVerifier('pencil', salt).split('$').slice(1);
  assert.equal(head, `4096:${salt.toString('base64')}`);
  const salted = pbkdf2Sync('pencil', salt, 4096, 32, 'sha256');
  const stored = createHash('sha256').update(createHmac('sha256', salted).update('Client Key').digest()).digest('base64');
  const server = createHmac('sha256', salted).update('Server Key').digest('base64');
  assert.equal(keys, `${stored}:${server}`);
});

test('Ollama is installed only for a local Ollama provider', () => {
  assert.deepEqual(triagePlan({}), { ollama: true, model: 'llama3.2' });
  assert.deepEqual(triagePlan({ INTAKE_LLM_BASE_URL: 'http://127.0.0.1:11434/v1', INTAKE_LLM_MODEL: 'qwen3' }), { ollama: true, model: 'qwen3' });
  assert.deepEqual(triagePlan({ INTAKE_LLM_BASE_URL: 'http://gpu-box:11434/v1' }), { ollama: false });
  for (const provider of ['openai', 'off', 'fake', 'llamacpp']) assert.deepEqual(triagePlan({ INTAKE_LLM_PROVIDER: provider }), { ollama: false });
  assert.throws(() => triagePlan({ INTAKE_LLM_BASE_URL: 'http://ollama:11434/v1' }), /Docker Compose host/);
});
