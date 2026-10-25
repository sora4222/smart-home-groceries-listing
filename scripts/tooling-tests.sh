#!/bin/sh
# Run repository-tooling suites through the same Make entrypoint in CI and locally.
set -eu
cd "$(dirname "$0")/.."
for suite in service-deployment compose-config desktop environment grocery-env test-database fill-env \
  check-file-length check-migrations database-profiles; do
  sh "scripts/$suite.test.sh"
done
node --test scripts/environment-contract.test.mjs scripts/compose.test.mjs \
  scripts/dev.test.mjs scripts/dev-context.test.mjs scripts/process.test.mjs \
  scripts/service-env.test.mjs \
  scripts/op-environment.test.mjs scripts/deploy-setup.test.mjs \
  scripts/migration-plan.test.mjs scripts/build-frontend.test.mjs
