/** Run one Compose command against this worktree's development project. */
import { DEVELOPMENT_FILES, loadDevelopmentContext } from './dev-context.mjs';
import { cancellation } from './test-process.mjs';
import { runCompose } from './compose.mjs';

const [action, ...args] = process.argv.slice(2);
try {
  await runCompose({ action, args, files: DEVELOPMENT_FILES, overrides: loadDevelopmentContext(), signal: cancellation() });
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
