#!/usr/bin/env bash
# Install reviewed local control code, then configure/start from a local bundle or existing install.
set -euo pipefail
[[ $(id -u) == 0 ]] || { echo 'Run with sudo; secrets belong in environment variables or protected input files' >&2; exit 1; }
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bash "$root/deploy/service/bootstrap.sh"
exec flock --nonblock /run/grocery-install.lock /usr/bin/node /usr/local/lib/grocery-service/deploy/service/start.mjs "$@"
