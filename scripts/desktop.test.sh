#!/bin/sh
# Desktop launch tests do not need Tauri, 1Password or a server.
set -eu
node --test scripts/desktop.test.mjs
