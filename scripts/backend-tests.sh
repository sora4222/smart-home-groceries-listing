#!/bin/sh
# Run with a disposable Docker database; never read application credentials.
set -eu
cd "$(dirname "$0")/.."
exec node scripts/backend-tests.mjs "$@"
