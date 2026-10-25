/** A bounded, data-only release format. No archive links, owners or install hooks. */
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve, sep } from 'node:path';
import { gzipSync, gunzipSync } from 'node:zlib';

export const MAX_BUNDLE = 256 * 1024 * 1024;
export const MAX_EXPANDED = 768 * 1024 * 1024;
export const digest = data => createHash('sha256').update(data).digest('hex');
const allowed = path => path === 'VERSION' || path === 'bin/grocery-backend'
  || ['frontend/', 'alexa-bridge/', 'migrations/'].some(prefix => path.startsWith(prefix));

/** Validate all entries before creating any destination files. */
export function decodeBundle(bytes) {
  if (bytes.length > MAX_BUNDLE) throw new Error('Release exceeds compressed size limit');
  const bundle = JSON.parse(gunzipSync(bytes, { maxOutputLength: MAX_EXPANDED }));
  if (bundle.schema !== 1 || !/^[a-f0-9]{40}$/.test(bundle.commit)
    || !/^[A-Za-z0-9._-]{1,100}$/.test(bundle.version)
    || !/^linux-(x64|arm64)$/.test(bundle.platform)
    || !/^3\.\d+$/.test(bundle.python) || !Array.isArray(bundle.files)
    || bundle.files.length < 2 || bundle.files.length > 30000) throw new Error('Invalid release metadata');
  const seen = new Set();
  let size = 0;
  for (const file of bundle.files) {
    if (typeof file.path !== 'string' || !/^[A-Za-z0-9._@+/-]+$/.test(file.path)
      || file.path.split('/').some(part => !part || part === '.' || part === '..' || part.startsWith('.env'))
      || !allowed(file.path) || seen.has(file.path) || ![0o644, 0o755].includes(file.mode)
      || typeof file.data !== 'string' || !/^[a-f0-9]{64}$/.test(file.sha256)) throw new Error('Invalid release entry');
    const data = Buffer.from(file.data, 'base64');
    if (data.toString('base64') !== file.data || digest(data) !== file.sha256) throw new Error('Release entry checksum mismatch');
    size += data.length;
    if (size > MAX_EXPANDED) throw new Error('Release exceeds expanded size limit');
    seen.add(file.path);
  }
  for (const path of seen) {
    let parent = dirname(path);
    while (parent !== '.') {
      if (seen.has(parent)) throw new Error('Release file is also a directory');
      parent = dirname(parent);
    }
  }
  if (!seen.has('bin/grocery-backend') || !seen.has('frontend/.output/server/index.mjs')) throw new Error('Incomplete release');
  return bundle;
}

/** Package only runtime components, never configuration, helpers or unit files. */
export function packBundle(source, metadata) {
  const files = [];
  const base = realpathSync(source);
  function walk(path, depth = 0) {
    if (depth > 40 || files.length >= 30000) throw new Error('Release tree too large');
    const full = join(base, path);
    const info = lstatSync(full);
    if (info.isSymbolicLink()) {
      // Interpreter links are recreated to the OS interpreter by the installer.
      if (/^alexa-bridge\/venv\/(bin\/python[^/]*|lib64)$/.test(path)) return;
      const target = realpathSync(full);
      if (!target.startsWith(base + sep)) throw new Error('Release symlink escapes runtime tree');
    }
    const real = realpathSync(full);
    const actual = lstatSync(real);
    if (actual.isDirectory()) {
      for (const child of readdirSync(real).sort()) walk(path + '/' + child, depth + 1);
    } else if (actual.isFile()) {
      const data = readFileSync(full);
      files.push({ path, mode: actual.mode & 0o111 ? 0o755 : 0o644, sha256: digest(data), data: data.toString('base64') });
    } else throw new Error('Release contains a special file');
  }
  for (const path of ['bin/grocery-backend', 'frontend', 'alexa-bridge', 'migrations', 'VERSION']) {
    if (existsSync(join(base, path))) walk(path);
  }
  const bytes = gzipSync(JSON.stringify({ schema: 1, ...metadata, files }), { level: 6 });
  decodeBundle(bytes);
  return bytes;
}

/** Extract a validated release only into a newly created, private directory. */
export function extractBundle(bundle, destination) {
  mkdirSync(destination, { mode: 0o755 }); // Refuse pre-existing paths and symlinks.
  for (const file of bundle.files) {
    const path = resolve(destination, file.path);
    mkdirSync(dirname(path), { recursive: true, mode: 0o755 });
    writeFileSync(path, Buffer.from(file.data, 'base64'), { flag: 'wx', mode: file.mode });
  }
}
