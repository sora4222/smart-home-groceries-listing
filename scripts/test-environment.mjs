/** Retain toolchain settings while excluding application credentials from test servers. */
export function testEnvironment(inherited = process.env) {
  return Object.fromEntries(Object.entries(inherited).filter(([key]) =>
    /^(PATH|HOME|USER|TMPDIR|TMP|TEMP|SystemRoot|CI|TERM|LANG|PNPM_HOME|E2E_CHROMIUM_PATH)$/.test(key)
    || /^(CARGO_|RUSTUP_|RUSTFLAGS$|PLAYWRIGHT_|LC_)/.test(key)));
}
