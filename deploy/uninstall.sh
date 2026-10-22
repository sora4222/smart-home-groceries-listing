#!/usr/bin/env bash
# `sudo make hm-uninstall-server`: stop and remove the grocery services and program
# files. Keeps /etc/grocery, the PostgreSQL database, the access logs in
# /var/log/grocery, and the Ollama and cloudflared packages.
set -euo pipefail

[ "$(id -u)" -eq 0 ] || { echo "run with sudo: sudo make hm-uninstall-server" >&2; exit 1; }
units=/etc/systemd/system

systemctl disable --now grocery.target grocery-backend.service grocery-alexa-bridge.service \
  grocery-frontend.service grocery-tunnel.service > /dev/null 2>&1 || true
rm -f "$units"/grocery-backend.service "$units"/grocery-alexa-bridge.service \
  "$units"/grocery-frontend.service "$units"/grocery-tunnel.service "$units"/grocery.target
rm -rf "$units/grocery-.service.d" /opt/grocery /opt/grocery.new /opt/grocery.old /run/grocery-secrets
systemctl daemon-reload
echo "removed; kept /etc/grocery, the database and /var/log/grocery"
