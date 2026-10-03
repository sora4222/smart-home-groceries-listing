#!/bin/sh
# Tests for scripts/check-env-files.sh. Run with `make test-tooling`.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
fails=0

check() {
    if [ "$2" = "$3" ]; then
        printf 'ok   %s\n' "$1"
    else
        printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$1" "$2" "$3"
        fails=$((fails + 1))
    fi
}

run() { printf '%s\n' "$@" | sh "$here/check-env-files.sh" && echo pass || echo fail; }

check "the template is allowed" "pass" "$(run .env.example)"
check "a dot environment file fails" "fail" "$(run .env | tail -n 1)"
check "environment variants fail" "fail" "$(run .env.production | tail -n 1)"
check "the plain environment file fails" "fail" "$(run env | tail -n 1)"
check "nested environment files fail" "fail" "$(run frontend/.env.local | tail -n 1)"
check "ordinary files pass" "pass" "$(run docs/using-the-app.md)"

[ "$fails" -eq 0 ]
