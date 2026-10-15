#!/bin/sh
# Tests for scripts/check-db-profile.sh. Run with `make test-tooling`.
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

# A copy of the script in a fixture root, so the developer's real .env is
# never read.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/scripts"
cp "$here/check-db-profile.sh" "$tmp/scripts/"

# Prints "pass" or "fail" for the fixture script under a clean environment.
run() {
    if env -i PATH="$PATH" "$@" sh "$tmp/scripts/check-db-profile.sh" 2>/dev/null; then
        echo pass
    else
        echo fail
    fi
}

docker_url=postgres://grocery:pw@db:5432/grocery
host_url=postgres://grocery:pw@host.docker.internal:5432/grocery

check "docker database with the db profile" pass \
    "$(run DATABASE_URL=$docker_url COMPOSE_PROFILES=ollama,db)"
check "docker database without the db profile" fail \
    "$(run DATABASE_URL=$docker_url COMPOSE_PROFILES=ollama)"
check "docker database with no profiles at all" fail \
    "$(run DATABASE_URL=$docker_url)"
check "a profile merely containing 'db' does not count" fail \
    "$(run DATABASE_URL=$docker_url COMPOSE_PROFILES=dbx)"
check "server database without the db profile" pass \
    "$(run DATABASE_URL=$host_url COMPOSE_PROFILES=ollama)"

printf 'DATABASE_URL=%s\nCOMPOSE_PROFILES=ollama\n' "$docker_url" > "$tmp/.env"
check "reads an old .env without db" fail "$(run)"
check "the environment overrides .env, as in Compose" pass \
    "$(run COMPOSE_PROFILES=db,ollama)"

[ "$fails" -eq 0 ]
