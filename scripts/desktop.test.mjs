/** Desktop launch configuration, independent of household services. */
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { desktopContext, desktopEnvironment } from './desktop-context.mjs';

test('development identifiers isolate worktrees while builds retain the installed identity', () => {
  const first = desktopContext('/checkout/one', 'dev');
  assert.deepEqual(first, desktopContext('/checkout/one', 'dev'));
  assert.notEqual(first.identifier, desktopContext('/checkout/two', 'dev').identifier);
  assert.equal(desktopContext('/checkout/one', 'build'), null);
  assert.throws(() => desktopContext('/checkout/one', 'other'), /dev or build/);
});

test('desktop processes receive their public URL and toolchain environment only', () => {
  const env = desktopEnvironment({
    PATH: '/bin', HOME: '/home/developer', CARGO_HOME: '/cache/cargo',
    OP_SERVICE_ACCOUNT_TOKEN: 'bootstrap', DATABASE_URL: 'private',
    CLERK_SECRET_KEY: 'private', VITE_CLERK_PUBLISHABLE_KEY: 'unused',
    STORE_TAB_SECRET: 'private', GROCERY_ENV_FILE: '/private/config',
    GROCERY_DESKTOP_SERVER_URL: 'https://groceries.example.test',
  });
  assert.deepEqual(env, {
    PATH: '/bin', HOME: '/home/developer', CARGO_HOME: '/cache/cargo',
    GROCERY_DESKTOP_SERVER_URL: 'https://groceries.example.test',
  });
});

test('server defaults are one public HTTP origin and never carry credentials', async () => {
  const { desktopServerUrl } = await import('./desktop-context.mjs');
  assert.equal(desktopServerUrl(' https://Groceries.tailnet.ts.net/ '), 'https://groceries.tailnet.ts.net');
  assert.equal(desktopServerUrl('http://100.91.154.126:3000'), 'http://100.91.154.126:3000');
  assert.equal(desktopServerUrl(''), '');
  for (const value of ['https://user:secret@example.test', 'https://*.example.test',
    'https://example.test/order', 'https://example.test/?token=private', 'file:///etc/passwd']) {
    assert.throws(() => desktopServerUrl(value), /server origin/);
  }
});

test('the real launcher resolves only the selected public setting and strips secrets', async () => {
  const { mkdtempSync, writeFileSync, rmSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { execFileSync } = await import('node:child_process');
  const dir = mkdtempSync(join(tmpdir(), 'grocery-desktop-launch-'));
  try {
    const file = join(dir, 'selected.env');
    writeFileSync(file, 'GROCERY_DESKTOP_SERVER_URL=https://groceries.test/\nDATABASE_URL=op://not-to-be-resolved\n');
    writeFileSync(join(dir, 'pnpm'), '#!/usr/bin/env node\nconsole.log(JSON.stringify({args:process.argv.slice(2),keys:Object.keys(process.env),url:process.env.GROCERY_DESKTOP_SERVER_URL}))\n', { mode: 0o700 });
    const output = execFileSync(process.execPath, [new URL('./desktop.mjs', import.meta.url).pathname, 'dev'], {
      encoding: 'utf8', env: { PATH: `${dir}:${process.env.PATH}`, HOME: process.env.HOME,
        GROCERY_ENV_FILE: file, OP_SERVICE_ACCOUNT_TOKEN: 'private', CLERK_SECRET_KEY: 'private' },
    });
    const result = JSON.parse(output);
    assert.equal(result.url, 'https://groceries.test');
    assert.deepEqual(result.args.slice(0, 3), ['exec', 'tauri', 'dev']);
    assert.match(JSON.parse(result.args[4]).identifier, /^com\.sora4222\.grocerylist\.dev\.w/);
    for (const key of ['OP_SERVICE_ACCOUNT_TOKEN', 'CLERK_SECRET_KEY', 'DATABASE_URL', 'GROCERY_ENV_FILE']) {
      assert.equal(result.keys.includes(key), false, key);
    }
  } finally { rmSync(dir, { recursive: true }); }
});
