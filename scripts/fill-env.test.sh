#!/bin/sh
# Tests for scripts/fill-env.sh. Run with `make test-tooling`.
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

# A fixture root with the script and a small .env.example, so the
# developer's real .env is never touched.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/scripts"
cp "$here/fill-env.sh" "$tmp/scripts/"
cat > "$tmp/.env.example" <<'EXAMPLE'
POSTGRES_USER=grocery
POSTGRES_PASSWORD=changeme
DATABASE_URL=postgres://grocery:changeme@db:5432/grocery
VOICE_WEBHOOK_SECRET=
ALEXA_BRIDGE_SECRET=
# a comment that must survive
STORE_TAB_SECRET=
CREDENTIAL_ENCRYPTION_KEY=
CLERK_SECRET_KEY=
EXAMPLE

run() { sh "$tmp/scripts/fill-env.sh"; }
value() { sed -n "s/^$1=//p" "$tmp/.env"; }

# 1. No .env yet: it is made from the example and every secret is filled.
output=$(run)
check "creates .env from the example" "yes" "$([ -f "$tmp/.env" ] && echo yes || echo no)"
check "fills the webhook secret (64 hex characters)" "64" "$(value VOICE_WEBHOOK_SECRET | tr -d '\n' | wc -c | tr -d ' ')"
check "fills the encryption key (base64 of 32 bytes)" "44" "$(value CREDENTIAL_ENCRYPTION_KEY | tr -d '\n' | wc -c | tr -d ' ')"
check "gives each secret its own value" "no" \
    "$([ "$(value ALEXA_BRIDGE_SECRET)" = "$(value VOICE_WEBHOOK_SECRET)" ] && echo yes || echo no)"
check "replaces the example database password" "no" "$([ "$(value POSTGRES_PASSWORD)" = changeme ] && echo yes || echo no)"
check "puts the same password in DATABASE_URL" \
    "postgres://grocery:$(value POSTGRES_PASSWORD)@db:5432/grocery" "$(value DATABASE_URL)"
check "leaves values it cannot make (Clerk) empty" "" "$(value CLERK_SECRET_KEY)"
check "keeps comments" "1" "$(grep -c '^# a comment that must survive$' "$tmp/.env")"
check "never prints a secret value" "no" \
    "$(printf '%s' "$output" | grep -q "$(value STORE_TAB_SECRET)" && echo yes || echo no)"

# 2. Run again: nothing already set is changed.
before=$(cat "$tmp/.env")
run > /dev/null
check "a second run changes nothing" "$before" "$(cat "$tmp/.env")"

# 3. An existing (older) .env keeps its database password, even 'changeme': the
#    database may already have been created with it.
printf 'POSTGRES_PASSWORD=changeme\nDATABASE_URL=postgres://grocery:changeme@db:5432/grocery\nSTORE_TAB_SECRET=\nVOICE_WEBHOOK_SECRET=keepme\n' > "$tmp/.env"
run > /dev/null
check "keeps an existing database password" "changeme" "$(value POSTGRES_PASSWORD)"
check "keeps a secret that is already set" "keepme" "$(value VOICE_WEBHOOK_SECRET)"
check "fills a secret that is still empty" "64" "$(value STORE_TAB_SECRET | tr -d '\n' | wc -c | tr -d ' ')"
check "adds a secret an older .env does not have yet" "64" "$(value ALEXA_BRIDGE_SECRET | tr -d '\n' | wc -c | tr -d ' ')"
check "adds each missing secret once" "1" "$(grep -c '^ALEXA_BRIDGE_SECRET=' "$tmp/.env")"
run > /dev/null
check "a later run adds it no second time" "1" "$(grep -c '^ALEXA_BRIDGE_SECRET=' "$tmp/.env")"

[ "$fails" -eq 0 ]
