#!/usr/bin/env node
/** Create a configuration-free runtime bundle from an already built application. */
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { packBundle } from './bundle.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const [source = root + 'build/production', output = root + 'build/grocery-linux-' + process.arch + '.json.gz'] = process.argv.slice(2);
try {
  const commit = execFileSync('git', ['-C', root, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  const python = execFileSync('/usr/bin/python3', ['-c', 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")'], { encoding: 'utf8' }).trim();
  writeFileSync(output, packBundle(source, { commit, python, platform: process.platform + '-' + process.arch,
    version: process.env.GROCERY_RELEASE_VERSION || commit }));
  console.log('Runtime bundle created; no server configuration included');
} catch (error) { console.error(error.message); process.exitCode = 1; }
