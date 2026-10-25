/** Back up and inspect the application database without exposing connection secrets. */
import { createWriteStream, closeSync, openSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { databaseTarget, setupSql } from '../../scripts/database-setup-sql.mjs';
import { migrationPlan } from '../../scripts/migration-plan.mjs';
import { readdirSync } from 'node:fs';
import { command, systemEnvironment } from './process.mjs';

/** PostgreSQL environment for an application-role connection; never put the URL in argv. */
export function postgresEnvironment(url) {
  let value;
  try { value = new URL(url); } catch { throw new Error('Invalid DATABASE_URL'); }
  if (!['postgres:', 'postgresql:'].includes(value.protocol)) throw new Error('Invalid database protocol');
  return { ...systemEnvironment, PGHOST: value.hostname, PGPORT: value.port || '5432',
    PGUSER: decodeURIComponent(value.username), PGPASSWORD: decodeURIComponent(value.password),
    PGDATABASE: decodeURIComponent(value.pathname.slice(1)), PGCONNECT_TIMEOUT: '10' };
}

/** Provision a local application role only during fresh installation. */
export function provisionDatabase(url) {
  const target = databaseTarget(url);
  if (target.local) command('runuser', ['-u', 'postgres', '--', 'psql', '-Xq', '-v', 'ON_ERROR_STOP=1'],
    { input: setupSql(target), env: { ...systemEnvironment, PGPORT: new URL(url).port || '5432' } });
}

/** Reject destructive pending migrations and schema downgrades before running new code. */
export function inspectMigrations(directory, url) {
  const env = postgresEnvironment(url);
  const present = command('psql', ['-XAtqc', "select to_regclass('public._sqlx_migrations') is not null"], { env }).trim();
  const applied = present === 't' ? command('psql', ['-XAtqc', 'select version from _sqlx_migrations where success'], { env }).trim().split(/\s+/).filter(Boolean) : [];
  const plan = migrationPlan(readdirSync(directory), applied, file => readFileSync(join(directory, file), 'utf8'));
  if (plan.ahead.length || plan.pending.some(item => item.destructive.length)) throw new Error('Migration would remove data or downgrade the schema; deployment stopped');
  return plan;
}

/** Write an exclusive, root-only PostgreSQL dump. Failure stops installation. */
export function backupDatabase(directory, url, commit) {
  const path = join(directory, new Date().toISOString().replaceAll(':', '-') + '-' + commit + '.dump');
  const fd = openSync(path, 'wx', 0o600);
  try {
    const result = spawnSync('pg_dump', ['--format=custom', '--no-owner'], {
      env: postgresEnvironment(url), cwd: '/', stdio: ['ignore', fd, 'pipe'], timeout: 300000,
    });
    if (result.error || result.status !== 0) throw new Error('Database backup failed; deployment stopped');
  } finally { closeSync(fd); }
  return path;
}
