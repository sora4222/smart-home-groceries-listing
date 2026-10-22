#!/usr/bin/env bash
# Install cloudflared from Cloudflare's signed apt repository, so the
# server's scheduled apt upgrades keep it current. The grocery-tunnel unit
# runs it; no cloudflared service of its own is installed.
set -euo pipefail

if command -v cloudflared > /dev/null; then
  echo "cloudflared $(cloudflared --version 2> /dev/null | head -n 1) already installed"
  exit 0
fi

[ "${ALLOW_DEPENDENCY_INSTALL:-}" = 1 ] || {
  echo 'cloudflared is missing. Install it first or explicitly allow installation with ALLOW_DEPENDENCY_INSTALL=1.' >&2
  exit 1
}

keyring=/usr/share/keyrings/cloudflare-main.gpg
key=$(mktemp)
trap 'rm -f "$key"' EXIT
curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg -o "$key"
install -d -m 755 /usr/share/keyrings
install -m 644 "$key" "$keyring"
cat > /etc/apt/sources.list.d/cloudflared.sources << SOURCES
Types: deb
URIs: https://pkg.cloudflare.com/cloudflared
Suites: any
Components: main
Architectures: $(dpkg --print-architecture)
Signed-By: $keyring
SOURCES
chmod 644 /etc/apt/sources.list.d/cloudflared.sources
apt-get update -qq
apt-get install -y -qq cloudflared
