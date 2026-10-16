#!/bin/sh
# Run configuration-loader and Compose contract tests from any directory.
set -eu
cd "$(dirname "$0")/.."
node --test scripts/environment.test.mjs
