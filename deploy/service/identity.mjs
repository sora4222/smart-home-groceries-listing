/** Detect accidental database/key changes across unattended updates without storing secrets twice. */
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { protectedWrite } from './configuration.mjs';

const path = '/var/lib/grocery-service/identity.json';
const hash = value => createHash('sha256').update(value).digest('hex');

/** Password rotation leaves the database identity intact; moving data or keys does not. */
export function dataIdentity(settings) {
  const url = new URL(settings.DATABASE_URL);
  return { database: hash(JSON.stringify([url.hostname, url.port || '5432', url.pathname, url.username])),
    encryption: hash(settings.CREDENTIAL_ENCRYPTION_KEY || '') };
}

export function assertDataIdentity(settings, file = path) {
  const current = dataIdentity(settings);
  if (existsSync(file) && JSON.stringify(JSON.parse(readFileSync(file, 'utf8'))) !== JSON.stringify(current)) {
    throw new Error('Database target or encryption key differs from the installed data identity; migrate deliberately');
  }
}

export function recordDataIdentity(settings, file = path) {
  assertDataIdentity(settings, file);
  protectedWrite(file, JSON.stringify(dataIdentity(settings)) + '\n');
}
