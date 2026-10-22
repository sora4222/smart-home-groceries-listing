#!/bin/sh
# Exercise Make target composition and lifecycle guards without application data.
set -eu
exec node --test "$(dirname "$0")/makefile.test.mjs"
