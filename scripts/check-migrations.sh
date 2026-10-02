#!/bin/sh
# Fails when a pull request edits, renames or deletes a migration that
# already exists. sqlx stores each applied migration's checksum, so changing
# one breaks every database that already ran it. Add a new migration instead
# (`make migration-new name="..."`).
#
# Reads `git diff --name-status` output on stdin.
set -eu

fails=0
while IFS="$(printf '\t')" read -r status path rest; do
    case "$status" in A*) continue ;; esac
    case "$path" in
        backend/migrations/*.sql)
            printf '%s: existing migration changed (%s)\n' "$path" "$status"
            fails=$((fails + 1))
            ;;
    esac
done

[ "$fails" -eq 0 ]
