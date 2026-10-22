/** Worktree identity and the desktop toolchain's environment boundary. */
import { createHash } from 'node:crypto';

/** Development instances have separate settings, cookies and single-instance locks. */
export function desktopContext(root, action) {
  if (action === 'build') return null;
  if (action !== 'dev') throw new Error('Expected dev or build');
  const suffix = createHash('sha256').update(root).digest('hex').slice(0, 12);
  return { identifier: `com.sora4222.grocerylist.dev.w${suffix}`, productName: 'Grocery List Dev' };
}

const TOOLCHAIN_KEYS = new Set([
  'PATH', 'HOME', 'USER', 'LOGNAME', 'SHELL', 'TMPDIR', 'TMP', 'TEMP',
  'SystemRoot', 'SYSTEMROOT', 'COMSPEC', 'PATHEXT', 'APPDATA', 'LOCALAPPDATA',
  'USERPROFILE', 'HOMEDRIVE', 'HOMEPATH', 'PROGRAMFILES', 'ProgramFiles',
  'LANG', 'DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XDG_CONFIG_HOME',
  'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'DBUS_SESSION_BUS_ADDRESS',
  'CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'RUSTUP_HOME',
  'RUSTUP_TOOLCHAIN', 'RUSTFLAGS', 'SDKROOT', 'DEVELOPER_DIR', 'MACOSX_DEPLOYMENT_TARGET',
  'CC', 'CXX', 'AR', 'PKG_CONFIG_PATH', 'GROCERY_DESKTOP_SERVER_URL',
]);

/** Never pass backend, store or 1Password credentials to Cargo, Tauri or the app. */
export function desktopEnvironment(values) {
  return Object.fromEntries(Object.entries(values).filter(([key, value]) =>
    value !== undefined && (TOOLCHAIN_KEYS.has(key) || key.startsWith('LC_'))));
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
