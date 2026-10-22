/** Stable worktree ownership and local addresses shared by startup and shutdown. */
import { createHash } from 'node:crypto';
import { realpathSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

/** Explicit settings override the defaults; collisions fail rather than reuse a port. */
export function developmentContext(env, root = fileURLToPath(new URL('../', import.meta.url))) {
  const hash = createHash('sha256').update(realpathSync(root)).digest('hex');
  const base = 20000 + (Number.parseInt(hash.slice(0, 6), 16) % 15000) * 2;
  const port = (key, fallback) => {
    const value = env[key] || String(fallback);
    if (!/^\d+$/.test(value) || Number(value) < 1024 || Number(value) > 65535) throw new Error(`Invalid ${key}`);
    return value;
  };
  return {
    ...env,
    COMPOSE_PROJECT_NAME: env.COMPOSE_PROJECT_NAME || `grocery-dev-${hash.slice(0, 12)}`,
    WEB_PORT: port('WEB_PORT', base), BACKEND_PORT: port('BACKEND_PORT', base + 1),
  };
}
