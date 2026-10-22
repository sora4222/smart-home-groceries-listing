#!/bin/sh
# Validate production and watched-development Compose contracts without application secrets.
set -eu
root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
umask 077
fixture=$(mktemp)
trap 'rm -f "$fixture"' EXIT HUP INT TERM

# Use the committed example and an empty environment, never a household .env.
compose() {
  profiles=$1
  shift
  env -i PATH="$PATH" HOME="$HOME" COMPOSE_PROFILES="$profiles" \
    WEB_PORT=43001 BACKEND_PORT=43002 docker compose \
    --env-file "$root/.env.example" -f "$root/docker-compose.yml" "$@"
}

compose db config --quiet
compose db config --services | grep -qx db
compose ollama config --quiet
compose db -f "$root/docker-compose.dev.yml" config --format json > "$fixture"
node --input-type=module - "$fixture" <<'JS'
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const { services } = JSON.parse(readFileSync(process.argv[2], 'utf8'));
assert.equal(services.frontend.environment.SERVER_API_BASE_URL, 'http://backend:8000');
assert.equal(services.frontend.environment.VITE_API_BASE_URL, 'http://localhost:43002');
assert.equal(services.frontend.ports[0].published, '43001');
assert.equal(services.frontend.ports[0].host_ip, '127.0.0.1');
assert.equal(services.backend.ports[0].published, '43002');
assert.equal(services.backend.ports[0].host_ip, '127.0.0.1');
assert.equal(services.db.ports?.length ?? 0, 0);
assert.ok(services.frontend.develop.watch.some(rule => rule.action === 'sync' && rule.initial_sync));
console.log('compose: production profiles and isolated watched-development configuration passed');
JS
