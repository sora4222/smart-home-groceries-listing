/**
 * Which configuration keys each production service may receive.
 *
 * Each systemd service gets only its own keys, so a leak from one process
 * (the internet-facing Alexa bridge, say) does not expose another's secrets.
 * Listening addresses are fixed in the unit files, not here, so `.env` cannot
 * move a service off loopback by accident.
 */

/** Backend keys, matching `backend/src/environment/mod.rs` minus unit-owned ones. */
const BACKEND_KEYS = [
  'DATABASE_URL', 'DATABASE_MAX_CONNECTIONS', 'POSTGRES_USER', 'POSTGRES_PASSWORD',
  'CLERK_JWKS_URL', 'VOICE_WEBHOOK_SECRET', 'ALEXA_BRIDGE_SECRET', 'STORE_TAB_SECRET',
  'CREDENTIAL_ENCRYPTION_KEY', 'CORS_ORIGINS', 'DEV_AUTH_BYPASS', 'STORE_CLIENTS',
  'WOOLWORTHS_BASE_URL', 'COLES_BASE_URL', 'STORE_TIMEOUT_SECONDS', 'STORE_SEARCH_CACHE_SECONDS',
  'INTAKE_LLM_PROVIDER', 'INTAKE_LLM_BASE_URL', 'INTAKE_LLM_MODEL', 'INTAKE_LLM_API_KEY',
  'OPENAI_API_KEY', 'INTAKE_LLM_MIN_CONFIDENCE', 'INTAKE_LLM_TIMEOUT_SECONDS',
  'GOOGLE_TASKS_CLIENT', 'GOOGLE_CLIENT_ID', 'GOOGLE_CLIENT_SECRET', 'GOOGLE_REDIRECT_URI',
  'GOOGLE_OAUTH_AUTH_URL', 'GOOGLE_OAUTH_TOKEN_URL', 'GOOGLE_TASKS_API_BASE',
  'GOOGLE_TIMEOUT_SECONDS', 'GOOGLE_TASKS_NO_BACKGROUND_POLL', 'RUST_LOG',
];

/**
 * `keys` or `prefixes` select what a service may read; `rename` maps a `.env`
 * name to the name the program expects. `database` and `triage` are not
 * services: `make install` reads them to set up PostgreSQL and Ollama.
 */
export const COMPONENTS = {
  backend: { keys: BACKEND_KEYS },
  'alexa-bridge': { keys: ['ALEXA_SKILL_ID', 'ALEXA_BRIDGE_SECRET', 'BACKEND_TIMEOUT_SECONDS', 'LOG_LEVEL'] },
  frontend: { prefixes: ['CLERK_', 'VITE_'] },
  tunnel: { keys: ['CLOUDFLARE_TUNNEL_TOKEN'], rename: { CLOUDFLARE_TUNNEL_TOKEN: 'TUNNEL_TOKEN' } },
  database: { keys: ['DATABASE_URL'] },
  triage: { keys: ['INTAKE_LLM_PROVIDER', 'INTAKE_LLM_BASE_URL', 'INTAKE_LLM_MODEL'] },
};

/** Look up a component, failing with the list of valid names. */
export function component(name) {
  const found = Object.hasOwn(COMPONENTS, name) ? COMPONENTS[name] : undefined;
  if (!found) throw new Error(`Unknown component "${name}"; expected one of: ${Object.keys(COMPONENTS).join(', ')}`);
  return found;
}

/** True when `key` belongs to the component. Never accepts 1Password's own `OP_*` keys. */
export function accepts(spec, key) {
  if (key.startsWith('OP_')) return false;
  return (spec.keys ?? []).includes(key) || (spec.prefixes ?? []).some(prefix => key.startsWith(prefix));
}

/** Apply the component's renames to already-resolved values. */
export function renamed(spec, values) {
  const result = {};
  for (const [key, value] of Object.entries(values)) result[spec.rename?.[key] ?? key] = value;
  return result;
}
