#!/bin/sh
# Fails when a change adds a real environment file. `.env.example` is the
# intentionally committed template; every other `.env` variant, plus `env`,
# can contain credentials and must remain untracked.
#
# Reads changed file paths on stdin, one per line.
set -eu

fails=0
while IFS= read -r path; do
    case "$path" in
        env | */env | .env | */.env | .env.* | */.env.*)
            case "$path" in
                .env.example | */.env.example) continue ;;
            esac
            printf '%s: environment files must not be committed\n' "$path"
            fails=$((fails + 1))
            ;;
    esac
done

[ "$fails" -eq 0 ]
