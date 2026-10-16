/** Select the Compose database service from a resolved PostgreSQL URL. */
import { fileURLToPath } from 'node:url';

/** Keep model profiles while enabling Docker PostgreSQL only for the db hostname. */
export function databaseProfiles(databaseUrl, configured = '') {
  let url;
  try { url = new URL(databaseUrl); }
  catch { throw new Error('DATABASE_URL must be a PostgreSQL connection URL'); }
  if (!['postgres:', 'postgresql:'].includes(url.protocol)) {
    throw new Error('DATABASE_URL must be a PostgreSQL connection URL');
  }
  const profiles = new Set(configured.split(',').map(value => value.trim()).filter(value => value && value !== 'db'));
  if (url.hostname === 'db') profiles.add('db');
  return [...profiles].join(',');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try { console.log(databaseProfiles(process.env.DATABASE_URL, process.env.COMPOSE_PROFILES)); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
