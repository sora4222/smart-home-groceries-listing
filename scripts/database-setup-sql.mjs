/**
 * Turn the resolved DATABASE_URL into idempotent SQL that creates (or
 * updates) the application role and database on this server's PostgreSQL.
 *
 *   grocery-env json database | node database-setup-sql.mjs | runuser -u postgres -- psql
 *   grocery-env json database | node database-setup-sql.mjs --print-database   →  "local grocery" | "remote"
 *
 * The password is sent as a SCRAM-SHA-256 verifier computed here, so the
 * plain password never appears in PostgreSQL's statement log. A URL that
 * names another server prints nothing: that database is managed elsewhere.
 */
import { createHash, createHmac, pbkdf2Sync, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

/** Hostnames left over from the Docker Compose setup. */
const DOCKER_HOSTS = new Set(['db', 'host.docker.internal']);
const LOCAL_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1']);
const ITERATIONS = 4096;

/** PostgreSQL's SCRAM-SHA-256 verifier for `password` (as stored in pg_authid). */
export function scramVerifier(password, salt = randomBytes(16)) {
  const salted = pbkdf2Sync(password.normalize('NFKC'), salt, ITERATIONS, 32, 'sha256');
  const clientKey = createHmac('sha256', salted).update('Client Key').digest();
  const storedKey = createHash('sha256').update(clientKey).digest();
  const serverKey = createHmac('sha256', salted).update('Server Key').digest();
  return `SCRAM-SHA-256$${ITERATIONS}:${salt.toString('base64')}$${storedKey.toString('base64')}:${serverKey.toString('base64')}`;
}

const literal = value => `'${value.replaceAll("'", "''")}'`;
const identifier = value => `"${value.replaceAll('"', '""')}"`;

/** Parse the URL into what the setup needs. Errors never include the password. */
export function databaseTarget(url) {
  let parsed;
  try { parsed = new URL(url); } catch { throw new Error('DATABASE_URL is not a valid URL'); }
  const host = parsed.hostname;
  if (DOCKER_HOSTS.has(host)) {
    throw new Error(`DATABASE_URL names "${host}", a Docker Compose host; use localhost for this server's PostgreSQL`);
  }
  return {
    local: LOCAL_HOSTS.has(host),
    user: decodeURIComponent(parsed.username),
    password: decodeURIComponent(parsed.password),
    database: decodeURIComponent(parsed.pathname.replace(/^\//, '')),
  };
}

/** The SQL for psql. Re-running it only refreshes the password. */
export function setupSql({ user, password, database }, verifier = scramVerifier) {
  if (!user || !database) throw new Error('DATABASE_URL must name a user and a database');
  if (user === 'postgres') throw new Error('DATABASE_URL must use an application role, not postgres');
  const credential = password ? ` PASSWORD ${literal(verifier(password))}` : '';
  return [
    'SET log_statement = none;',
    `SELECT format('CREATE ROLE %I', ${literal(user)}) WHERE NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = ${literal(user)})\\gexec`,
    `ALTER ROLE ${identifier(user)} WITH LOGIN${credential};`,
    `SELECT format('CREATE DATABASE %I OWNER %I', ${literal(database)}, ${literal(user)}) WHERE NOT EXISTS (SELECT FROM pg_catalog.pg_database WHERE datname = ${literal(database)})\\gexec`,
    '',
  ].join('\n');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const { DATABASE_URL: url } = JSON.parse(readFileSync(0, 'utf8'));
    if (!url) throw new Error('DATABASE_URL is not set in .env');
    const target = databaseTarget(url);
    if (process.argv[2] === '--print-database') process.stdout.write(target.local ? `local ${target.database}\n` : 'remote\n');
    else if (target.local) process.stdout.write(setupSql(target));
    else console.error('DATABASE_URL names another server; skipping database setup');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
