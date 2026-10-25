#!/bin/sh
# Unit coverage for the fixed native release installer and optional startup configuration.
set -eu
cd "$(dirname "$0")/.."
node --test deploy/service/service.test.mjs
