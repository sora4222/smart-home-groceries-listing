/** Worktree identity and the desktop toolchain's environment boundary. */
import { createHash } from 'node:crypto';
import { toolchainEnvironment } from './toolchain-environment.mjs';

/** Development instances have separate settings, cookies and single-instance locks. */
export function desktopContext(root, action) {
  if (action === 'build') return null;
  if (action !== 'dev') throw new Error('Expected dev or build');
  const suffix = createHash('sha256').update(root).digest('hex').slice(0, 12);
  return { identifier: `com.sora4222.grocerylist.dev.w${suffix}`, productName: 'Grocery List Dev' };
}

/** Native tools receive the shared toolchain settings and one public URL. */
export function desktopEnvironment(values) {
  const env = toolchainEnvironment(values);
  if (values.GROCERY_DESKTOP_SERVER_URL !== undefined) {
    env.GROCERY_DESKTOP_SERVER_URL = values.GROCERY_DESKTOP_SERVER_URL;
  }
  return env;
}

/** Validate the optional public default before it can be embedded in a binary. */
export function desktopServerUrl(input = '') {
  if (!input.trim()) return '';
  const problem = () => new Error('GROCERY_DESKTOP_SERVER_URL must be one HTTP(S) server origin without credentials, wildcards, paths or queries');
  let url;
  try { url = new URL(input.trim()); } catch { throw problem(); }
  if (!['http:', 'https:'].includes(url.protocol) || !url.hostname ||
    url.hostname.includes('*') || url.username || url.password ||
    url.pathname !== '/' || url.search || url.hash) throw problem();
  return url.origin;
}
