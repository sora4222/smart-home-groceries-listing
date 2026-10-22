#!/bin/sh
# Tests for deploy/bin/grocery-env, the root-side configuration resolver the
# systemd units run before each service starts. Run with `make test-tooling`.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
fails=0

# check NAME EXPECTED ACTUAL
check() {
    if [ "$2" = "$3" ]; then
        printf 'ok   %s\n' "$1"
    else
        printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$1" "$2" "$3"
        fails=$((fails + 1))
    fi
}

# The installed layout (/opt/grocery) in miniature, with a fake `op` that
# answers only when the service-account token reached it.
tmp=$(mktemp -d)
# Node resolves module entrypoints through macOS /var symlinks.
tmp=$(CDPATH='' cd "$tmp" && pwd -P)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/opt/bin" "$tmp/opt/lib" "$tmp/opt/libexec" "$tmp/path" "$tmp/etc"
cp "$here/../deploy/bin/grocery-env" "$tmp/opt/libexec/"
cp "$here"/environment.mjs "$here"/service-env.mjs "$here"/service-components.mjs "$here"/systemd-env.mjs "$tmp/opt/lib/"
ln -s "$(command -v node)" "$tmp/opt/bin/node"
cat > "$tmp/path/op" <<'OP'
#!/bin/sh
[ -n "${OP_SERVICE_ACCOUNT_TOKEN:-}" ] || exit 1
printf 'resolved-%s' "${3##*/}"
OP
chmod +x "$tmp/path/op"
cat > "$tmp/etc/grocery.env" <<'ENV'
ALEXA_SKILL_ID=op://Vault/item/skill
ALEXA_BRIDGE_SECRET=literal$secret
DATABASE_URL=postgres://grocery:pw@localhost/grocery
ENV
printf 'fixture-token\n' > "$tmp/etc/token"

run() {
    GROCERY_CONFIG="$tmp/etc/grocery.env" GROCERY_ETC="$tmp/etc" \
    GROCERY_OP_TOKENS="$tmp/etc/missing.token $tmp/etc/token" \
    GROCERY_SECRETS_DIR="$tmp/run" GROCERY_PATH="$tmp/path:/usr/bin:/bin" \
    OP_SERVICE_ACCOUNT_TOKEN=exported-must-be-ignored \
        sh "$tmp/opt/libexec/grocery-env" "$@"
}

run write alexa-bridge > /dev/null
# shellcheck disable=SC2016 # the dollar sign is the literal being checked
check "writes only the component's keys, escaped for systemd" \
    "$(printf 'ALEXA_BRIDGE_SECRET="literal\\$secret"\nALEXA_SKILL_ID="resolved-skill"')" \
    "$(sort "$tmp/run/alexa-bridge.env")"
check "resolved file is private to root" "600" "$(node -e 'console.log((require("node:fs").statSync(process.argv[1]).mode & 0o777).toString(8))' "$tmp/run/alexa-bridge.env")"
check "secrets directory is private" "700" "$(node -e 'console.log((require("node:fs").statSync(process.argv[1]).mode & 0o777).toString(8))' "$tmp/run")"
check "no temporary files are left" "alexa-bridge.env" "$(ls -A "$tmp/run")"
check "json output for install steps" \
    '{"DATABASE_URL":"postgres://grocery:pw@localhost/grocery"}' "$(run json database)"

# The exported token is not used: without a token file, the reference fails.
rm "$tmp/etc/token"
status=0
run write alexa-bridge > /dev/null 2>&1 || status=$?
check "an exported token never authorises the resolver" "1" "$status"
status=0
run write nonsense > /dev/null 2>&1 || status=$?
check "unknown components are refused" "1" "$status"

if [ "$fails" -gt 0 ]; then
    echo "$fails grocery-env test(s) failed"
    exit 1
fi
