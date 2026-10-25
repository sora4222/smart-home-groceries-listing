/** Resolve installed component settings through the fixed, root-owned helper. */
import { command } from './process.mjs';

export function runtimeSettings(component) {
  return JSON.parse(command('/usr/local/lib/grocery-service/libexec/grocery-env', ['json', component]));
}

/** Validate that an effective database/key change would not strand existing data. */
export function unchangedIdentity(previous, next) {
  const identity = url => { const value = new URL(url); return [value.hostname, value.port || '5432', value.pathname, value.username].join('|'); };
  if (previous.DATABASE_URL && identity(previous.DATABASE_URL) !== identity(next.DATABASE_URL)) throw new Error('Existing DATABASE_URL target changed; migrate it separately');
  if (previous.CREDENTIAL_ENCRYPTION_KEY && previous.CREDENTIAL_ENCRYPTION_KEY !== next.CREDENTIAL_ENCRYPTION_KEY) throw new Error('Existing CREDENTIAL_ENCRYPTION_KEY changed; migrate it separately');
}
