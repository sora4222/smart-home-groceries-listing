/** Disposable PostgreSQL for host-side tests, independent of application settings. */
import { randomBytes } from 'node:crypto';
import { PostgreSqlContainer } from '@testcontainers/postgresql';

/** Bind the randomly allocated database port only to local test clients. */
class LocalPostgres extends PostgreSqlContainer {
  async beforeContainerCreated() {
    this.hostConfig.PortBindings['5432/tcp'][0].HostIp = '127.0.0.1';
  }
}

/** Start a ready PostgreSQL 16 server without a persistent application volume. */
export async function startPostgres() {
  return new LocalPostgres('postgres:16')
    .withDatabase('postgres')
    .withUsername('grocery_test')
    .withPassword(randomBytes(24).toString('hex'))
    .withLabels({ 'grocery.purpose': 'test' })
    .start();
}

/** Own a fresh database server for exactly one action, including failed actions. */
export async function withTestDatabase(action, { start = startPostgres } = {}) {
  const database = await start();
  try { return await action(database.getConnectionUri()); }
  finally { await database.stop(); }
}
