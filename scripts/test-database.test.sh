#!/bin/sh
# Exercise test-database lifecycle without Docker or application credentials.
set -eu
cd "$(dirname "$0")/.."
node --test scripts/test-database.test.mjs
