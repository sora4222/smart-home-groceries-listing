#!/usr/bin/env bash
# `sudo make install`: put the output of `make build` into service on this
# server. Safe to re-run; each run is also how an update is deployed.
#
#   1. .env               -> /etc/grocery/grocery.env (root only)
#   2. build/production   -> /opt/grocery (root owned, read-only to services)
#   3. PostgreSQL role and database from DATABASE_URL (local server only)
#   4. Ollama, when it is the triage provider on this machine
#   5. cloudflared, when CLOUDFLARE_TUNNEL_TOKEN is set
#   6. systemd units, enabled at boot and (re)started
#
# Optional: OP_DESKTOP_USER=<login> resolves op:// values through that
# user's 1Password desktop app when no service-account token is installed.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
staged=$root/build/production
prefix=/opt/grocery
etc=/etc/grocery
units=/etc/systemd/system
services=(grocery-backend grocery-alexa-bridge grocery-frontend)

log() { printf '==> %s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
die() { printf 'make install: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "run with sudo: sudo make install"
[ -x "$staged/bin/grocery-backend" ] || die "nothing built yet; run 'make build' as your own user first"
[ -f "$root/.env" ] || die "no .env in $root; run 'make setup-env' and fill it in (docs/production.md)"
node=$(command -v node) || die "node is not on root's PATH; install Node 22 system-wide"
case $node in
  /home/* | /root/*) die "node is inside a home directory ($node); services cannot read it there. Install Node system-wide" ;;
esac
command -v op > /dev/null || warn "1Password CLI (op) not found; op:// values in .env cannot be resolved"

log "configuration -> $etc/grocery.env"
install -d -m 700 -o root -g root "$etc"
install -m 600 -o root -g root "$root/.env" "$etc/grocery.env"
if [ -n "${OP_DESKTOP_USER:-}" ]; then
  id -u "$OP_DESKTOP_USER" > /dev/null 2>&1 || die "OP_DESKTOP_USER $OP_DESKTOP_USER does not exist"
  printf '%s\n' "$OP_DESKTOP_USER" > "$etc/op-desktop-user"
  chmod 600 "$etc/op-desktop-user"
fi

log "program files -> $prefix ($(cat "$staged/VERSION"))"
rm -rf "$prefix.new"
cp -a "$staged" "$prefix.new"
ln -sfn "$node" "$prefix.new/bin/node"
chown -R root:root "$prefix.new"
chmod -R u+rwX,go+rX,go-w "$prefix.new"

# Reads one component's resolved settings as JSON, through the new resolver.
settings() { "$prefix.new/libexec/grocery-env" json "$1"; }
lib=$prefix.new/lib

log "PostgreSQL role and database"
if systemctl is-active --quiet postgresql; then
  settings database | "$node" "$lib/database-setup-sql.mjs" \
    | (cd / && runuser -u postgres -- psql -X -q -v ON_ERROR_STOP=1)
else
  warn "postgresql.service is not running; skipped creating the role and database"
fi

log "intake triage model"
plan=$(settings triage | "$node" "$lib/triage-setup.mjs")
if [[ $plan == ollama\ * ]]; then
  bash "$root/deploy/install-ollama.sh" "${plan#ollama }"
else
  echo "no local Ollama needed"
fi

log "Cloudflare Tunnel"
has_tunnel=$(settings tunnel | "$node" -e \
  'process.stdout.write(JSON.parse(require("fs").readFileSync(0, "utf8")).TUNNEL_TOKEN ? "yes" : "no")')
if [ "$has_tunnel" = yes ]; then
  bash "$root/deploy/install-cloudflared.sh"
  services+=(grocery-tunnel)
else
  warn "CLOUDFLARE_TUNNEL_TOKEN is empty; the tunnel is not started, so Alexa cannot reach the bridge"
  systemctl disable --now grocery-tunnel.service > /dev/null 2>&1 || true
fi

log "systemd units"
install -m 644 "$prefix.new"/systemd/*.service "$prefix.new/systemd/grocery.target" "$units/"
install -d -m 755 "$units/grocery-.service.d"
install -m 644 "$prefix.new"/systemd/grocery-.service.d/*.conf "$units/grocery-.service.d/"
rm -rf "$prefix.old"
if [ -d "$prefix" ]; then mv "$prefix" "$prefix.old"; fi
mv "$prefix.new" "$prefix"
systemctl daemon-reload
systemctl enable --quiet grocery.target "${services[@]/%/.service}"

log "starting ${services[*]}"
if ! systemctl restart "${services[@]/%/.service}"; then
  systemctl --no-pager --lines=20 status "${services[@]/%/.service}" || true
  die "a service did not start; see: journalctl -u 'grocery-*' -e"
fi
rm -rf "$prefix.old"

for _ in $(seq 30); do
  if BIND_ADDRESS=127.0.0.1:8000 "$prefix/bin/grocery-backend" --health-check > /dev/null 2>&1; then
    log "backend is healthy; the web app is at http://localhost:3000"
    exit 0
  fi
  sleep 1
done
die "the backend did not report healthy within 30 seconds; see: journalctl -u grocery-backend -e"
