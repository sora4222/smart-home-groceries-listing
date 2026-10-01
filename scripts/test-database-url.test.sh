#!/bin/sh
# Tests for scripts/test-database-url.sh. Run with `make test-tooling`.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
script="$here/test-database-url.sh"
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

# A copy of the script in a fixture root, so the developer's real .env is
# never read. Tests that need a .env write one into "$tmp".
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/scripts"
cp "$script" "$tmp/scripts/"
fixture="$tmp/scripts/test-database-url.sh"

# Runs the fixture script with a clean environment.
run() { env -i PATH="$PATH" "$@" sh "$fixture"; }

check "uses localhost, never the compose host 'db'" \
    "postgres://grocery:secret@localhost:5432/postgres" \
    "$(run POSTGRES_PASSWORD=secret)"

check "encodes reserved characters in the password" \
    "postgres://grocery:a%40b%3Ac%2Fd%23e%25f@localhost:5432/postgres" \
    "$(run 'POSTGRES_PASSWORD=a@b:c/d#e%f')"

check "honours user, host and port overrides" \
    "postgres://me:pw@example.test:6543/postgres" \
    "$(run POSTGRES_USER=me POSTGRES_PASSWORD=pw POSTGRES_HOST=example.test POSTGRES_PORT=6543)"

printf 'DATABASE_URL=postgres://x:y@db:5432/grocery\nPOSTGRES_PASSWORD=from@dotenv\n' > "$tmp/.env"
check "reads the password from .env and ignores its DATABASE_URL" \
    "postgres://grocery:from%40dotenv@localhost:5432/postgres" \
    "$(run)"

check "the environment wins over .env" \
    "postgres://grocery:envwins@localhost:5432/postgres" \
    "$(run POSTGRES_PASSWORD=envwins)"

printf 'POSTGRES_PASSWORD=pw\nPOSTGRES_PORT=5532\n' > "$tmp/.env"
check "reads the published port from .env, as docker compose does" \
    "postgres://grocery:pw@localhost:5532/postgres" \
    "$(run)"

[ "$fails" -eq 0 ]
