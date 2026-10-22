#!/usr/bin/env bash
# `make build`: compile every production component into build/production,
# laid out exactly as `make hm-install-server` copies it to /opt/grocery. Runs as your
# own user, using sudo only to resolve frontend settings; no Docker.
#
#   bin/grocery-backend      Rust backend (release, migrations embedded)
#   alexa-bridge/venv        relocatable Python virtualenv with the bridge
#   frontend/.output         TanStack Start server bundle
#   lib/ libexec/ systemd/   configuration resolver and unit files
#   migrations/              copy of the SQL the binary embeds, for checks
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out=$root/build/production
stage=$out.tmp
python=${GROCERY_PYTHON:-/usr/bin/python3}
target=${CARGO_TARGET_DIR:-$root/backend/target}

log() { printf '==> %s\n' "$*"; }
die() { printf 'make build: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -ne 0 ] || die "run as your own user, not root (then: sudo make hm-install-server)"
missing=()
for tool in cargo node pnpm uv git cmake clang pkg-config; do
  command -v "$tool" > /dev/null || missing+=("$tool")
done
[ -x "$python" ] || missing+=("$python")
if [ "${#missing[@]}" -gt 0 ]; then
  die "missing ${missing[*]}. On Ubuntu: sudo apt install build-essential cmake clang libclang-dev pkg-config perl git python3; Rust, pnpm and uv as in docs/production.md"
fi
[ -f "$root/.env" ] || die "no .env; the web app build reads its VITE_* values from it (make setup-env)"

rm -rf "$stage"
mkdir -p "$stage/bin" "$stage/lib" "$stage/libexec" "$stage/alexa-bridge" "$stage/frontend"
trap 'rm -rf "$stage"' EXIT

log "backend: cargo build --release"
cargo build --release --locked --quiet --manifest-path "$root/backend/Cargo.toml"
install -m 755 "$target/release/grocery-backend" "$stage/bin/grocery-backend"

log "alexa bridge: locked dependencies into a virtualenv ($("$python" --version))"
export UV_PYTHON_DOWNLOADS=never
uv venv --quiet --relocatable --python "$python" "$stage/alexa-bridge/venv"
UV_PROJECT_ENVIRONMENT=$stage/alexa-bridge/venv uv sync --quiet --frozen --no-dev \
  --no-editable --compile-bytecode --python "$python" --project "$root/sidecars/alexa-bridge"
"$stage/alexa-bridge/venv/bin/python" -c 'import alexa_bridge.app, gunicorn' \
  || die "the alexa bridge virtualenv is incomplete"

log "web app: pnpm build (frontend settings resolved with the server 1Password account)"
pnpm --dir "$root/frontend" install --frozen-lockfile --silent
node "$root/scripts/build-frontend.mjs"
cp -a "$root/frontend/.output" "$stage/frontend/.output"

log "configuration resolver and systemd units"
for file in environment.mjs op-auth.mjs op-environment.mjs service-env.mjs service-components.mjs systemd-env.mjs \
  database-setup-sql.mjs triage-setup.mjs migration-plan.mjs run-with-env.mjs; do
  install -m 644 "$root/scripts/$file" "$stage/lib/$file"
done
install -m 755 "$root/deploy/bin/grocery-env" "$stage/libexec/grocery-env"
# The migrations the binary embeds, so `make hm-update-server` can check them first.
cp -a "$root/backend/migrations" "$stage/migrations"
cp -a "$root/deploy/systemd" "$stage/systemd"
git -C "$root" describe --always --dirty > "$stage/VERSION" 2> /dev/null || echo unknown > "$stage/VERSION"

rm -rf "$out"
mv "$stage" "$out"
trap - EXIT
log "built $(cat "$out/VERSION") into build/production; next: sudo make hm-install-server"
