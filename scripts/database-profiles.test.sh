#!/bin/sh
# Check runtime database mode without application settings or Docker services.
set -eu
cd "$(dirname "$0")/.."
node --test scripts/database-profiles.test.mjs
