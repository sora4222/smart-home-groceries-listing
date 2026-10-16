#!/bin/sh
# Resolve test-database credentials before Cargo; preserve every failure status.
set -eu
cd "$(dirname "$0")/.."
database_url=${TEST_DATABASE_URL:-}
if [ -z "$database_url" ]; then
    database_url=$(node scripts/run-env.mjs --prefix POSTGRES_ sh scripts/test-database-url.sh)
fi
cd backend
if [ "${1:-}" != --compact ]; then
    export DATABASE_URL="$database_url"
    exec cargo test "$@"
fi

log=$(mktemp "${TMPDIR:-/tmp}/grocery-backend-tests.XXXXXX")
if DATABASE_URL="$database_url" cargo test --quiet > "$log" 2>&1; then
    awk '/^test result:/ { passed += $4; failed += $6 }
        END { printf "backend: %d passed, %d failed\n", passed, failed }' "$log"
    rm -f "$log"
else
    tail -n 40 "$log"
    printf 'Backend tests failed; full log: %s\n' "$log" >&2
    exit 1
fi
