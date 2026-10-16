#!/bin/sh
# Validate the selected database profile, then preserve Compose's signal handling.
set -eu
cd "$(dirname "$0")/.."
sh scripts/check-db-profile.sh
exec docker compose up "$@"
