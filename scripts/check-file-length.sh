#!/bin/sh
# Fails when a file is longer than AGENTS.md rule 1 allows:
# Rust and Python 300 lines, TypeScript 400.
#
# Reads file paths on stdin, one per line (CI passes the files a pull
# request adds or changes). Paths that no longer exist are skipped, and so
# is TanStack Router's generated route tree.
set -eu

fails=0
while IFS= read -r path; do
    [ -f "$path" ] || continue
    case "$path" in
        *routeTree.gen.ts) continue ;;
        *.rs | *.py) limit=300 ;;
        *.ts | *.tsx) limit=400 ;;
        *) continue ;;
    esac
    lines=$(wc -l < "$path" | tr -d ' ')
    if [ "$lines" -gt "$limit" ]; then
        printf '%s: %s lines (limit %s)\n' "$path" "$lines" "$limit"
        fails=$((fails + 1))
    fi
done

[ "$fails" -eq 0 ]
