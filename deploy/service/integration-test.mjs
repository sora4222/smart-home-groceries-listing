#!/usr/bin/env node
/** Disposable Ubuntu runner only: real PostgreSQL, systemd startup, updates and backup restore. */
import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';
import { decodeBundle, digest } from './bundle.mjs';
import { command, systemEnvironment } from './process.mjs';
import { postgresEnvironment } from './database.mjs';

if (process.env.GROCERY_DISPOSABLE_TEST !== '1' || process.getuid() !== 0) throw new Error('Run only on an explicitly disposable runner as root');
if (existsSync('/etc/grocery/grocery.env') || existsSync('/opt/grocery')) throw new Error('Refusing to test against an existing installation');
const root = fileURLToPath(new URL('../../', import.meta.url));
const bundlePath = resolve(process.argv[2]);
const mode = process.argv[3] || 'file';
const temporary = mkdtempSync('/run/grocery-install-ci.');
const env = { ...systemEnvironment, DATABASE_URL: 'postgresql://grocery_ci:fixture_password@localhost:5432/grocery_ci',
  CREDENTIAL_ENCRYPTION_KEY: Buffer.alloc(32, 7).toString('base64'), DEV_AUTH_BYPASS: 'true', VITE_AUTH_MODE: 'off',
  VITE_API_BASE_URL: 'http://127.0.0.1:8000', SERVER_API_BASE_URL: 'http://127.0.0.1:8000',
  STORE_CLIENTS: 'fake', GOOGLE_TASKS_CLIENT: 'fake', INTAKE_LLM_PROVIDER: 'off' };
const configuration = join(temporary, 'fixture.env');
writeFileSync(configuration, Object.entries(env).filter(([key]) => !['PATH', 'HOME', 'GROCERY_ENV_FILE'].includes(key)).map(([key, value]) => key + '=' + value).join('\n'), { mode: 0o600 });
const opMarker = join(temporary, 'op-called');
writeFileSync('/usr/local/bin/op', `#!/usr/bin/node\nimport('node:fs').then(fs=>{if(process.env.OP_SERVICE_ACCOUNT_TOKEN!=='fixture-token')process.exit(9);fs.writeFileSync('${opMarker}','called');process.stdout.write(JSON.stringify({...process.env,STORE_TIMEOUT_SECONDS:'17'}));});\n`, { mode: 0o755 });
const tokenFile = join(temporary, 'fixture.token');
writeFileSync(tokenFile, 'fixture-token\n', { mode: 0o600 });
const args = [join(root, 'deploy/start.sh'), '--bundle', bundlePath];
let startEnv = systemEnvironment;
if (mode === 'file' || mode === 'op-file') args.push('--config', configuration);
else startEnv = { ...env };
if (mode === 'op-file') args.push('--op-token-file', tokenFile, '--op-environment-id', 'fixture');
if (mode === 'op-env') startEnv = { ...startEnv, OP_SERVICE_ACCOUNT_TOKEN: 'fixture-token', OP_ENVIRONMENT_ID: 'fixture' };
command('bash', args, { env: startEnv, timeout: 600000, stdio: 'inherit' });
assert.equal(existsSync(opMarker), mode.startsWith('op-'));
const dbEnv = postgresEnvironment(env.DATABASE_URL);
command('psql', ['-Xq', '-v', 'ON_ERROR_STOP=1', '-c', "create table deployment_sentinel(id integer primary key, payload text); insert into deployment_sentinel values(1,'retain household information')"], { env: dbEnv });
const before = readFileSync('/etc/grocery/grocery.env');
command('/usr/local/sbin/grocery-install-release', [bundlePath], { timeout: 600000, stdio: 'inherit' });
assert.deepEqual(readFileSync('/etc/grocery/grocery.env'), before);
assert.equal(command('psql', ['-XAtqc', 'select payload from deployment_sentinel where id=1'], { env: dbEnv }).trim(), 'retain household information');
// A destructive migration is rejected before stopping the healthy application.
const bad = decodeBundle(readFileSync(bundlePath));
const data = Buffer.from('DROP TABLE deployment_sentinel;');
bad.files.push({ path: 'migrations/99999999_destroy.sql', mode: 0o644, sha256: digest(data), data: data.toString('base64') });
const badPath = join(temporary, 'destructive.json.gz');
writeFileSync(badPath, gzipSync(JSON.stringify(bad)));
assert.throws(() => command('/usr/local/sbin/grocery-install-release', [badPath], { timeout: 600000 }));
assert.equal(command('psql', ['-XAtqc', 'select count(*) from deployment_sentinel'], { env: dbEnv }).trim(), '1');
command('systemctl', ['is-enabled', '--quiet', 'grocery.target', 'grocery-backend.service', 'grocery-frontend.service']);
command('systemctl', ['stop', 'grocery.target']);
command('systemctl', ['start', 'grocery.target']);
command('systemctl', ['is-active', '--quiet', 'grocery-backend', 'grocery-frontend']);
const pid = command('systemctl', ['show', 'grocery-backend', '--property=MainPID', '--value']).trim();
assert.ok(!readFileSync('/proc/' + pid + '/environ', 'utf8').split('\0').some(value => value.startsWith('OP_')));
if (mode.startsWith('op-')) {
  assert.ok(readFileSync('/proc/' + pid + '/environ', 'utf8').includes('STORE_TIMEOUT_SECONDS=17'));
  assert.ok(!before.includes(Buffer.from('fixture-token')));
}
const backups = readdirSync('/var/backups/grocery').filter(file => file.endsWith('.dump')).sort();
assert.ok(backups.length >= 2);
command('runuser', ['-u', 'postgres', '--', 'createdb', '-O', 'grocery_ci', 'grocery_ci_restore']);
command('pg_restore', ['--no-owner', '--exit-on-error', '--dbname=grocery_ci_restore', '/var/backups/grocery/' + backups.at(-1)], { env: dbEnv });
assert.equal(command('psql', ['-XAtqc', 'select count(*) from deployment_sentinel'], { env: { ...dbEnv, PGDATABASE: 'grocery_ci_restore' } }).trim(), '1');
console.log('Passed fresh install, optional configuration, data-preserving update, destructive-migration rejection, boot enablement, restart, token exclusion and backup restore: ' + mode);
