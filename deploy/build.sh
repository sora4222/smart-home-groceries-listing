#!/usr/bin/env bash
# Assemble component outputs into the systemd install layout in build/production.
# Direct invocation delegates to Make so component builds share one dependency graph.
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
if [ "${1:-}" != --assemble ]; then exec make -C "$root" build; fi
out=$root/build/production
mkdir -p "$root/build"
stage=$(mktemp -d "$root/build/.production.XXXXXX")
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/bin" "$stage/lib" "$stage/libexec" "$stage/frontend"
install -m 755 "$root/build/backend/grocery-backend" "$stage/bin/grocery-backend"
cp -a "$root/build/alexa" "$stage/alexa-bridge"
cp -a "$root/build/frontend/.output" "$stage/frontend/.output"
for file in environment.mjs op-auth.mjs op-environment.mjs service-env.mjs service-components.mjs systemd-env.mjs \
  database-setup-sql.mjs triage-setup.mjs migration-plan.mjs run-with-env.mjs; do
  install -m 644 "$root/scripts/$file" "$stage/lib/$file"
done
install -m 755 "$root/deploy/bin/grocery-env" "$stage/libexec/grocery-env"
cp -a "$root/backend/migrations" "$stage/migrations"
cp -a "$root/deploy/systemd" "$stage/systemd"
git -C "$root" describe --always --dirty > "$stage/VERSION" 2> /dev/null || echo unknown > "$stage/VERSION"
rm -rf "$out"
mv "$stage" "$out"
trap - EXIT
printf 'built build/production; next: sudo make install-server\n'
