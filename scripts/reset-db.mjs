/** Explicitly reset only the PostgreSQL schema inside this worktree's Compose db. */
import { runCompose } from './compose.mjs';
import { DEVELOPMENT_FILES, loadDevelopmentContext } from './dev-context.mjs';
import { cancellation } from './test-process.mjs';
import { loadEnvironment, selectedEnvironmentFile } from './environment.mjs';

try {
  if (!process.argv.includes('--confirm')) throw new Error('Deletes this worktree Compose database data. Run make reset-db confirm=reset to confirm.');
  const overrides = loadDevelopmentContext();
  const values = loadEnvironment({ ...selectedEnvironmentFile(), accepts: key => key.startsWith('POSTGRES_') });
  const user = encodeURIComponent(values.POSTGRES_USER || 'grocery');
  const database = encodeURIComponent(values.POSTGRES_DB || 'grocery');
  if (!values.POSTGRES_PASSWORD) throw new Error('POSTGRES_PASSWORD is required to reset the local database');
  overrides.DATABASE_URL = `postgres://${user}:${encodeURIComponent(values.POSTGRES_PASSWORD)}@db:5432/${database}`;
  const context = { files: DEVELOPMENT_FILES, overrides, signal: cancellation() };
  console.log(`Resetting Compose database in ${overrides.COMPOSE_PROJECT_NAME}`);
  await runCompose({ ...context, action: 'stop', args: ['backend'] });
  // exec requires a running local db service; it cannot follow a remote DATABASE_URL.
  await runCompose({ ...context, action: 'exec', args: [
    '-T', 'db', 'sh', '-c',
    'exec psql -X -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"',
  ] });
  // Force the local database address, even when the app normally uses a remote server.
  await runCompose({ ...context, action: 'run', args: [
    '--rm', '--no-deps', 'backend', 'grocery-backend', '--migrate-only',
  ] });
  console.log('Database migrated. Run make dev to restart the complete stack.');
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
