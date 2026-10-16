#!/bin/sh
# Select the resolved database mode, then preserve Compose's signal handling.
set -eu
cd "$(dirname "$0")/.."
COMPOSE_PROFILES=$(node scripts/database-profiles.mjs)
export COMPOSE_PROFILES
exec docker compose up "$@"
