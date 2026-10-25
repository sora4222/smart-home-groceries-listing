/** Startup arguments contain file paths and identifiers; secret values come from files or exports. */
export function startOptions(argv, env = process.env) {
  const result = { environmentId: env.OP_ENVIRONMENT_ID, token: env.OP_SERVICE_ACCOUNT_TOKEN };
  const names = { '--config': 'config', '--op-token-file': 'tokenFile', '--op-environment-id': 'environmentId', '--bundle': 'bundle' };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--adopt-existing') result.adopt = true;
    else if (Object.hasOwn(names, argv[index]) && argv[index + 1] && !argv[index + 1].startsWith('--')) result[names[argv[index]]] = argv[++index];
    else throw new Error('Usage: start.sh [--bundle PATH|--adopt-existing] [--config PATH] [--op-token-file PATH] [--op-environment-id ID]');
  }
  if (result.environmentId && !/^[A-Za-z0-9_-]+$/.test(result.environmentId)) throw new Error('Invalid Environment ID');
  if (result.bundle && result.adopt) throw new Error('Choose a bundle or adopt the existing installation');
  return result;
}
