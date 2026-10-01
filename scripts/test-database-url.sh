#!/bin/sh
# Prints the DATABASE_URL the backend's integration tests should use.
#
# `.env` points DATABASE_URL at the Compose hostname `db`, which only resolves
# inside the Docker network. Tests run on the host, so they need the published
# port on localhost instead. Settings are read from the environment first, then
# from `.env`, then defaulted. `.env` is read with sed rather than sourced so a
# password containing `#`, `$` or `@` cannot be misread as shell.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)

# setting KEY DEFAULT — environment, else .env, else DEFAULT.
setting() {
    value=$(printenv "$1" || true)
    if [ -z "$value" ] && [ -f "$root/.env" ]; then
        value=$(sed -n "s/^$1=//p" "$root/.env" | head -n 1 | sed 's/^"\(.*\)"$/\1/')
    fi
    printf '%s' "${value:-$2}"
}

# Percent-encodes the characters that would end a URL's userinfo early.
encode() {
    printf '%s' "$1" | sed \
        -e 's/%/%25/g' -e 's/@/%40/g' -e 's/:/%3A/g' -e 's#/#%2F#g' \
        -e 's/?/%3F/g' -e 's/#/%23/g' -e 's/\$/%24/g' -e 's/ /%20/g'
}

user=$(encode "$(setting POSTGRES_USER grocery)")
password=$(encode "$(setting POSTGRES_PASSWORD "")")
host=$(setting POSTGRES_HOST localhost)
port=$(setting POSTGRES_PORT 5432)

# Database `postgres`: #[sqlx::test] creates a throwaway database per test, so
# it only needs a server it may create databases on.
printf 'postgres://%s:%s@%s:%s/postgres\n' "$user" "$password" "$host" "$port"
