#!/usr/bin/env bash
# Ubuntu prerequisites for installing runtime bundles; no build toolchain or Docker.
set -euo pipefail
[[ $(id -u) == 0 ]] || exit 1
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends nodejs python3 postgresql postgresql-client curl ca-certificates util-linux
node -e 'if(Number(process.versions.node.split(".")[0])<22)process.exit(1)'
systemctl enable --now postgresql.service
