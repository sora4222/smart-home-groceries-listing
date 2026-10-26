/** Own a disposable database and local-only test servers for the browser suite. */
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import { withTestDatabase } from './test-database.mjs';
import { cancellation, runProcess, startProcess, withServers } from './test-process.mjs';
import { readResults, verifyResults } from './test-results.mjs';
import { testEnvironment } from './test-environment.mjs';

/** Allocate distinct available ports without assuming a worktree's fixed ports. */
async function ports() {
  const servers = [createServer(), createServer()];
  try {
    return await Promise.all(servers.map(server => new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(0, '127.0.0.1', () => resolve(server.address().port));
    })));
  } finally {
    await Promise.all(servers.map(server => new Promise(resolve => server.close(resolve))));
  }
}

/** Wait for HTTP readiness, failing early if an owned server exits. */
async function ready(url, servers, signal) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    signal.throwIfAborted();
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(2000) });
      await response.body?.cancel();
      if (response.ok) return;
    } catch {}
    await delay(250, undefined, { signal });
  }
  throw new Error('Test server did not become ready within 120 seconds');
}

const signal = cancellation();
const args = process.argv.slice(2);
const human = args[0] === '--human';
if (human) args.shift();
const dir = mkdtempSync(join(tmpdir(), 'grocery-e2e-'));
const emptyEnv = join(dir, '.env');
writeFileSync(emptyEnv, '', { mode: 0o600 });
mkdirSync(join(dir, 'access-logs'));
const servers = [];
const backend = new URL('../backend/', import.meta.url);
const frontend = new URL('../frontend/', import.meta.url);
const base = testEnvironment();
try {
  await runProcess('cargo', ['build', '--locked', '--quiet'], { cwd: backend, env: base, log: join(dir, 'build.log'), signal });
  await withTestDatabase(async databaseUrl => {
    const [apiPort, webPort] = await ports();
    const apiUrl = `http://127.0.0.1:${apiPort}`;
    const webUrl = `http://127.0.0.1:${webPort}`;
    const env = {
      ...base, GROCERY_ENV_FILE: emptyEnv, DATABASE_URL: databaseUrl,
      BIND_ADDRESS: `127.0.0.1:${apiPort}`, CORS_ORIGINS: webUrl,
      DEV_AUTH_BYPASS: 'true', STORE_CLIENTS: 'fake', INTAKE_LLM_PROVIDER: 'fake',
      GOOGLE_TASKS_CLIENT: 'fake', GOOGLE_TASKS_NO_BACKGROUND_POLL: 'true',
      GOOGLE_REDIRECT_URI: `${webUrl}/settings/help`,
      VOICE_WEBHOOK_SECRET: 'e2e-webhook-secret', STORE_TAB_SECRET: 'e2e-store-tab-secret',
      CREDENTIAL_ENCRYPTION_KEY: Buffer.from(Array.from({ length: 32 }, (_, i) => i)).toString('base64'),
      ACCESS_LOG_DIR: join(dir, 'access-logs'),
      CLERK_SECRET_KEY: '', CLERK_PUBLISHABLE_KEY: '', CLERK_JWKS_URL: '',
      VITE_CLERK_PUBLISHABLE_KEY: '', VITE_AUTH_MODE: 'off',
      VITE_API_BASE_URL: apiUrl, VITE_WS_URL: `${apiUrl.replace('http:', 'ws:')}/ws`,
      E2E_API_BASE_URL: apiUrl, E2E_BASE_URL: webUrl,
    };
    try {
      servers.push(startProcess('cargo', ['run', '--locked', '--quiet', '--bin', 'grocery-backend'], {
        cwd: backend, env, log: join(dir, 'backend.log'), signal,
      }));
      await withServers(servers, serverSignal => ready(`${apiUrl}/api/health`, servers, AbortSignal.any([signal, serverSignal])));
      servers.push(startProcess('pnpm', ['exec', 'vite', 'dev', '--host', '127.0.0.1', '--port', String(webPort), '--strictPort'], {
        cwd: frontend, env, log: join(dir, 'frontend.log'), signal,
      }));
      await withServers(servers, serverSignal => ready(webUrl, servers, AbortSignal.any([signal, serverSignal])));
      console.log('e2e: isolated PostgreSQL, backend, and frontend ready');
      await withServers(servers, async serverSignal => {
        await runProcess('pnpm', ['e2e', human ? '--reporter=line,json' : '--reporter=json', ...args], {
          cwd: frontend, env: { ...env, PLAYWRIGHT_JSON_OUTPUT_FILE: join(dir, 'results.json') }, signal: AbortSignal.any([signal, serverSignal]), log: human ? undefined : join(dir, 'playwright.log'),
        });
        verifyResults('e2e', readResults('playwright', join(dir, 'results.json')));
      });
    } finally {
      await Promise.all(servers.map(server => server.stop()));
    }
  });
  rmSync(dir, { recursive: true });
} catch (error) {
  console.error(error.message);
  try { console.error(readFileSync(join(dir, 'playwright.log'), 'utf8').split('\n').slice(-40).join('\n')); } catch {}
  console.error(`E2E server logs: ${dir}`);
  process.exitCode ||= 1;
}
