#!/bin/sh
# Tests for scripts/check-file-length.sh. Run with `make test-tooling`.
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

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
# lines FILE COUNT — writes COUNT lines to FILE.
lines() { seq "$2" > "$tmp/$1"; }

run() { (cd "$tmp" && printf '%s\n' "$@" | sh "$here/check-file-length.sh" && echo pass || echo fail); }

lines ok.rs 300
lines long.rs 301
lines long.py 301
lines ok.tsx 400
lines long.ts 401
lines routeTree.gen.ts 900
lines notes.md 900

check "a Rust file at the limit passes" "pass" "$(run ok.rs)"
check "a TypeScript file at the limit passes" "pass" "$(run ok.tsx)"
check "a long Rust file fails" "long.rs: 301 lines (limit 300)
fail" "$(run long.rs)"
check "a long Python file fails" "fail" "$(run long.py | tail -n 1)"
check "a long TypeScript file fails" "fail" "$(run long.ts | tail -n 1)"
check "the generated route tree is skipped" "pass" "$(run routeTree.gen.ts)"
check "other file types are skipped" "pass" "$(run notes.md)"
check "a deleted file is skipped" "pass" "$(run gone.rs)"
check "one long file among several fails" "fail" "$(run ok.rs long.rs ok.tsx | tail -n 1)"

[ "$fails" -eq 0 ]
