/** Build with the caller's frontend configuration, without installed server credentials. */
import { fileURLToPath } from 'node:url';
import { loadEnvironment, selectedEnvironmentFile } from './environment.mjs';
import { accepts, component } from './service-components.mjs';
import { cancellation, runProcess } from './test-process.mjs';
import { toolchainEnvironment } from './toolchain-environment.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
try {
  const frontend = component('frontend');
  const values = loadEnvironment({
    ...selectedEnvironmentFile(),
    environmentId: process.env.OP_ENVIRONMENT_ID,
    accepts: key => accepts(frontend, key),
  });
  const env = toolchainEnvironment(process.env);
  Object.assign(env, values, { GROCERY_ENV_FILE: '/dev/null' });
  await runProcess('pnpm', ['--dir', `${root}frontend`, 'run', '--silent', 'build'], { env, signal: cancellation() });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
