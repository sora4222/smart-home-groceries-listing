/** Write resolved values in systemd's `EnvironmentFile=` format. */

/** Characters systemd treats specially inside a double-quoted value. */
const ESCAPED = /["\\`$]/g;

/**
 * One `KEY="value"` line per entry. systemd performs no variable expansion
 * here, but `"`, `\`, `` ` `` and `$` are escaped so the value reads back
 * exactly. Line breaks and other control characters cannot be represented
 * safely, so they are refused; the error names the key, never the value.
 */
export function formatSystemdEnvironment(values) {
  const lines = [];
  for (const [key, value] of Object.entries(values)) {
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) throw new Error(`Invalid variable name ${JSON.stringify(key)}`);
    // biome-ignore lint/suspicious/noControlCharactersInRegex: refusing control characters is the point.
    if (/[\u0000-\u001f\u007f]/.test(value)) throw new Error(`${key} contains a line break or control character`);
    lines.push(`${key}="${value.replace(ESCAPED, match => `\\${match}`)}"`);
  }
  return lines.length ? `${lines.join('\n')}\n` : '';
}
