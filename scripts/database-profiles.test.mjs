/** Runtime database choice must not depend on an obsolete Compose profile list. */
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { databaseProfiles } from './database-profiles.mjs';

for (const [url, profiles, expected] of [
  ['postgres://fixture@db:5432/grocery', 'ollama', 'ollama,db'],
  ['postgresql://db/grocery', '', 'db'],
  ['postgres://db', 'db,ollama,db', 'ollama,db'],
  ['postgres://fixture@localhost/grocery', 'db,ollama', 'ollama'],
  ['postgres://fixture@host.docker.internal:5432/grocery', 'db', ''],
  ['postgres://fixture@database.example/grocery', 'dbx, ollama', 'dbx,ollama'],
  ['postgres:///grocery?host=%2Fvar%2Frun%2Fpostgresql', 'db', ''],
]) {
  test(`selects profiles for ${new URL(url).hostname || 'Unix socket'} with ${profiles || 'no profiles'}`, () => {
    assert.equal(databaseProfiles(url, profiles), expected);
  });
}

test('invalid and unresolved URLs fail without exposing their value', () => {
  for (const value of ['secret-value', 'op://private/database/password', 'https://private.invalid']) {
    assert.throws(() => databaseProfiles(value), { message: 'DATABASE_URL must be a PostgreSQL connection URL' });
  }
});
