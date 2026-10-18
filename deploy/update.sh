#!/usr/bin/env bash
# Last step of `make update` (after deploy/pull.sh and `make build`), as root:
# check the database migrations the new build would apply, back up the
# database, then run `make install`, which migrates before switching over.
#
# Stops before touching anything when:
#   - a pending migration drops, truncates, deletes or retypes data
#     (re-run with ALLOW_DESTRUCTIVE=1 once you have read it), or
#   - the database has migrations this checkout does not know (an older
#     branch: its code would run against a newer schema).
# Backups go to /var/backups/grocery (root only, newest 10 kept).
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
staged=$root/build/production
backups=/var/backups/grocery
keep=10

log() { printf '==> %s\n' "$*"; }
die() { printf 'make update: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "run with sudo"
[ -d "$staged/migrations" ] || die "nothing built yet; run 'make build' first"
[ -f "$root/.env" ] || die "no .env in $root"
node=$(command -v node) || die "node is not on root's PATH"
lib=$staged/lib

# The staged resolver, reading the checkout's .env (install copies it later).
settings() { GROCERY_NODE=$node GROCERY_CONFIG=$root/.env "$staged/libexec/grocery-env" json "$1"; }
psql_db() { (cd / && runuser -u postgres -- psql -X -q -At -v ON_ERROR_STOP=1 -d "$1" "${@:2}"); }

read -r where database < <(settings database | "$node" "$lib/database-setup-sql.mjs" --print-database)
if [ "$where" != local ]; then
  log "DATABASE_URL names another server; skipping the migration check and backup"
elif ! systemctl is-active --quiet postgresql; then
  die "postgresql.service is not running"
elif [ "$(psql_db postgres -c "select count(*) from pg_database where datname = '${database//\'/\'\'}'")" = 0 ]; then
  log "database $database does not exist yet; make install creates it"
else
  log "checking migrations against database $database"
  applied=""
  if [ "$(psql_db "$database" -c "select to_regclass('public._sqlx_migrations') is not null")" = t ]; then
    applied=$(psql_db "$database" -c "select version from _sqlx_migrations where success")
  fi
  status=0
  "$node" "$lib/migration-plan.mjs" "$staged/migrations" <<< "$applied" || status=$?
  case $status in
    0) ;;
    3)
      [ "${ALLOW_DESTRUCTIVE:-}" = 1 ] \
        || die "a pending migration above can destroy data. Read it, then: make update ALLOW_DESTRUCTIVE=1" ;;
    4) die "the database is newer than this checkout (unknown migrations above); check out a newer branch" ;;
    *) die "could not check the migrations" ;;
  esac

  install -d -m 700 -o root -g root "$backups"
  file=$backups/$database-$(date +%Y%m%d-%H%M%S)-$(cat "$staged/VERSION").dump
  log "backing up $database to $file"
  (umask 077; cd / && runuser -u postgres -- pg_dump -Fc "$database") > "$file"
  chmod 600 "$file"
  # shellcheck disable=SC2012 # names are ours: no spaces or newlines
  ls -1t "$backups"/"$database"-*.dump | tail -n +$((keep + 1)) | xargs -r rm -f
fi

exec bash "$root/deploy/install.sh"
