/**
 * Decide whether `make update` may migrate the production database.
 *
 *   psql -Atc 'select version from _sqlx_migrations where success' \
 *     | node migration-plan.mjs MIGRATIONS_DIR
 *
 * Prints one line per pending migration and per destructive statement it
 * contains. Exit status: 0 safe to apply, 3 a pending migration may destroy
 * data, 4 the database is ahead of this checkout (an older branch).
 *
 * sqlx itself applies only pending files, in order, each in a transaction,
 * and refuses to run if an applied file was edited. This adds the checks it
 * does not make: data-destroying statements and a downgrade.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Statements that remove or rewrite stored data. Dropping an index or constraint does not. */
const DESTRUCTIVE = [
  ['DROP TABLE', /\bDROP\s+TABLE\b/i],
  ['DROP COLUMN', /\bDROP\s+COLUMN\b/i],
  ['DROP SCHEMA', /\bDROP\s+SCHEMA\b/i],
  ['DROP DATABASE', /\bDROP\s+DATABASE\b/i],
  ['TRUNCATE', /\bTRUNCATE\b/i],
  ['DELETE FROM', /\bDELETE\s+FROM\b/i],
  ['column type change', /\bALTER\s+COLUMN\s+\S+\s+(SET\s+DATA\s+)?TYPE\b/i],
];

/** The version sqlx records: the file name's numeric prefix. */
export function migrationVersion(file) {
  const match = /^(\d+)_.*\.sql$/.exec(file);
  return match ? BigInt(match[1]) : undefined;
}

/** SQL without comments or quoted text, so neither can trigger a finding. */
export function stripSql(sql) {
  return sql
    .replace(/\/\*[\s\S]*?\*\//g, ' ')
    .replace(/--[^\n]*/g, ' ')
    .replace(/\$([A-Za-z_]*)\$[\s\S]*?\$\1\$/g, ' ')
    .replace(/'(?:[^']|'')*'/g, "''");
}

/** Names of the destructive statement kinds in `sql`. */
export function destructiveStatements(sql) {
  const code = stripSql(sql);
  return DESTRUCTIVE.filter(([, pattern]) => pattern.test(code)).map(([name]) => name);
}

/** Compare the checkout's migrations with the versions the database has applied. */
export function migrationPlan(files, applied, read) {
  const known = new Map();
  for (const file of files) {
    const version = migrationVersion(file);
    if (version !== undefined) known.set(version, file);
  }
  const done = new Set(applied.map(value => BigInt(value)));
  const ahead = [...done].filter(version => !known.has(version)).sort((a, b) => (a < b ? -1 : 1));
  const pending = [...known].filter(([version]) => !done.has(version)).sort(([a], [b]) => (a < b ? -1 : 1));
  return {
    ahead: ahead.map(String),
    pending: pending.map(([, file]) => ({ file, destructive: destructiveStatements(read(file)) })),
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const dir = process.argv[2];
    if (!dir) throw new Error('Usage: migration-plan.mjs MIGRATIONS_DIR < applied-versions');
    const applied = readFileSync(0, 'utf8').split(/\s+/).filter(Boolean);
    const plan = migrationPlan(readdirSync(dir), applied, file => readFileSync(join(dir, file), 'utf8'));
    for (const version of plan.ahead) console.log(`unknown applied migration ${version}`);
    for (const { file, destructive } of plan.pending) {
      console.log(`pending ${file}`);
      for (const kind of destructive) console.log(`destructive ${file}: ${kind}`);
    }
    if (!plan.pending.length && !plan.ahead.length) console.log('database schema is up to date');
    if (plan.ahead.length) process.exitCode = 4;
    else if (plan.pending.some(({ destructive }) => destructive.length)) process.exitCode = 3;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
