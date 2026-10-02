#!/bin/sh
# Tests for scripts/check-migrations.sh. Run with `make test-tooling`.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
fails=0

# check NAME EXPECTED ACTUAL
check() {
    if [ "$2" = "$3" ]; then
        printf 'ok   %s\n' "$1"
    else
        printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$1" "$2" "$3"
        fails=$((fails + 1))
    fi
}

# run LINE... — feeds tab-separated `git diff --name-status` lines.
run() { printf '%s\n' "$@" | sh "$here/check-migrations.sh" >/dev/null && echo pass || echo fail; }

tab=$(printf '\t')
check "a new migration passes" "pass" "$(run "A${tab}backend/migrations/0008_new.sql")"
check "an edited migration fails" "fail" "$(run "M${tab}backend/migrations/0001_init.sql")"
check "a deleted migration fails" "fail" "$(run "D${tab}backend/migrations/0001_init.sql")"
check "a renamed migration fails" "fail" \
    "$(run "R100${tab}backend/migrations/0001_init.sql${tab}backend/migrations/0001_x.sql")"
check "an edited Rust file passes" "pass" "$(run "M${tab}backend/src/main.rs")"

[ "$fails" -eq 0 ]
