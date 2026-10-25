#!/usr/bin/env bash
# Install reviewed control code locally. Remote runtime bundles never update these files.
set -euo pipefail
[[ $(id -u) == 0 ]] || exit 1
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
fixed=/usr/local/lib/grocery-service
install -d -m 755 "$fixed" "$fixed/bin" "$fixed/libexec" "$fixed/scripts" "$fixed/deploy/service"
for file in environment.mjs op-auth.mjs op-environment.mjs service-env.mjs service-components.mjs systemd-env.mjs database-setup-sql.mjs migration-plan.mjs; do
  install -m 644 "$root/scripts/$file" "$fixed/scripts/$file"
done
install -m 644 "$root"/deploy/service/*.mjs "$fixed/deploy/service/"
ln -sfn scripts "$fixed/lib"
ln -sfn /usr/bin/node "$fixed/bin/node"
install -m 755 "$root/deploy/bin/grocery-env" "$fixed/libexec/grocery-env"
id grocery-migrate >/dev/null 2>&1 || useradd --system --no-create-home --shell /usr/sbin/nologin grocery-migrate
install -d -m 755 /etc/systemd/system/grocery-.service.d /usr/local/sbin
install -m 644 "$root"/deploy/systemd/*.service "$root/deploy/systemd/grocery.target" /etc/systemd/system/
install -m 644 "$root"/deploy/systemd/grocery-.service.d/*.conf /etc/systemd/system/grocery-.service.d/
# Bind resolver execution to this reviewed control installation, never the release tree.
for name in backend frontend alexa-bridge tunnel; do
  install -d -m 755 "/etc/systemd/system/grocery-$name.service.d"
  cat > "/etc/systemd/system/grocery-$name.service.d/20-fixed-resolver.conf" <<'UNIT'
[Service]
ExecStartPre=
ExecStartPre=+/usr/local/lib/grocery-service/libexec/grocery-env write COMPONENT
UNIT
  sed -i "s/COMPONENT/$name/" "/etc/systemd/system/grocery-$name.service.d/20-fixed-resolver.conf"
done
cat > /usr/local/sbin/grocery-install-release <<'SCRIPT'
#!/bin/sh
set -eu
exec flock --nonblock /run/grocery-install.lock /usr/bin/node /usr/local/lib/grocery-service/deploy/service/install-release.mjs "$@"
SCRIPT
chmod 755 /usr/local/sbin/grocery-install-release
systemctl daemon-reload
