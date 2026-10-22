/** Environment shared by native and frontend build subprocesses. */
const TOOLCHAIN_KEYS = new Set([
  'PATH', 'HOME', 'USER', 'LOGNAME', 'SHELL', 'TMPDIR', 'TMP', 'TEMP',
  'SystemRoot', 'SYSTEMROOT', 'COMSPEC', 'PATHEXT', 'APPDATA', 'LOCALAPPDATA',
  'USERPROFILE', 'HOMEDRIVE', 'HOMEPATH', 'PROGRAMFILES', 'ProgramFiles',
  'LANG', 'DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XDG_CONFIG_HOME',
  'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'DBUS_SESSION_BUS_ADDRESS',
  'CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'RUSTUP_HOME',
  'RUSTUP_TOOLCHAIN', 'RUSTFLAGS', 'SDKROOT', 'DEVELOPER_DIR', 'MACOSX_DEPLOYMENT_TARGET',
  'CC', 'CXX', 'AR', 'PKG_CONFIG_PATH', 'PNPM_HOME', 'CI', 'TERM', 'NO_COLOR',
]);

/** Pass OS and build-tool settings, excluding application and provider credentials. */
export function toolchainEnvironment(values) {
  return Object.fromEntries(Object.entries(values).filter(([key, value]) =>
    value !== undefined && (TOOLCHAIN_KEYS.has(key) || key.startsWith('LC_'))));
}
