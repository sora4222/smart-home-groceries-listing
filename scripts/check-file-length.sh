#!/bin/sh
# Enforces the existing 500-line repository gate for Rust, Python and TypeScript.
# Authoring instructions may impose stricter limits; this gate remains unchanged.
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
        *.rs | *.py) limit=500 ;;
        *.ts | *.tsx) limit=500 ;;
        *) continue ;;
    esac
    lines=$(wc -l < "$path" | tr -d ' ')
    if [ "$lines" -gt "$limit" ]; then
        printf '%s: %s lines (limit %s)\n' "$path" "$lines" "$limit"
        fails=$((fails + 1))
    fi
done

[ "$fails" -eq 0 ]
