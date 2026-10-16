#!/bin/sh
# Stops `make up` when DATABASE_URL names the Compose host `db` but the `db`
# profile is not in COMPOSE_PROFILES, so the Docker database would not start.
# Catches a `.env` made before the database moved into a profile, whose
# COMPOSE_PROFILES lacks `db`. Settings come from the environment first (as
# Compose reads them), then `.env`. Tested by scripts/check-db-profile.test.sh.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
env_file=${GROCERY_ENV_FILE:-$root/.env}

# setting KEY — environment, else .env, else nothing.
setting() {
    if value=$(printenv "$1"); then
        printf '%s' "$value"
        return
    fi
    if [ -f "$env_file" ]; then
        value=$(sed -n "s/^$1=//p" "$env_file" | head -n 1 | sed 's/^"\(.*\)"$/\1/')
    fi
    printf '%s' "$value"
}

url=$(setting DATABASE_URL)
profiles=",$(setting COMPOSE_PROFILES | tr -d ' '),"

case "$url" in
    *@db:* | *@db/*) ;;
    *) exit 0 ;;
esac

case "$profiles" in
    *,db,*) exit 0 ;;
esac

echo "DATABASE_URL points at the Docker database (db), but COMPOSE_PROFILES" >&2
echo "does not include db. Add it, e.g. COMPOSE_PROFILES=ollama,db, or point" >&2
echo "DATABASE_URL at a PostgreSQL on the server (docs/human-setup.md)." >&2
exit 1
