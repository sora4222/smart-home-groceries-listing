#!/bin/sh
# Makes `.env` ready for the values only a person can get.
#
# - No `.env` yet: copies `.env.example`, and gives the database a random
#   password (in POSTGRES_PASSWORD and DATABASE_URL, so they match).
# - Fills every app secret that is still empty with a new random value:
#   VOICE_WEBHOOK_SECRET, ALEXA_BRIDGE_SECRET, STORE_TAB_SECRET (hex) and
#   CREDENTIAL_ENCRYPTION_KEY (base64 of 32 bytes).
# - Never changes a value that is already set, and never changes the
#   database password of an existing `.env` (the database may already use it).
# - Prints the names it filled, never the values.
#
# Run it with `make setup-env`. Tested by scripts/fill-env.test.sh.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
env_file="$root/.env"

# set_value KEY VALUE — rewrites the KEY= line in .env.
set_value() {
    tmp_file="$env_file.tmp.$$"
    awk -v key="$1" -v value="$2" '
        index($0, key "=") == 1 { print key "=" value; next }
        { print }
    ' "$env_file" > "$tmp_file"
    mv "$tmp_file" "$env_file"
}

# current KEY — the value of KEY in .env, or nothing.
current() { sed -n "s/^$1=//p" "$env_file" | head -n 1; }

# has_key KEY — the line exists in .env.
has_key() { grep -q "^$1=" "$env_file"; }

if [ ! -f "$env_file" ]; then
    cp "$root/.env.example" "$env_file"
    chmod 600 "$env_file"
    echo "Made .env from .env.example"
    old_password=$(current POSTGRES_PASSWORD)
    new_password=$(openssl rand -hex 24)
    set_value POSTGRES_PASSWORD "$new_password"
    url=$(current DATABASE_URL)
    set_value DATABASE_URL "$(printf '%s' "$url" | sed "s#:$old_password@#:$new_password@#")"
    echo "Filled POSTGRES_PASSWORD and DATABASE_URL"
fi

for key in VOICE_WEBHOOK_SECRET ALEXA_BRIDGE_SECRET STORE_TAB_SECRET; do
    if has_key "$key" && [ -z "$(current "$key")" ]; then
        set_value "$key" "$(openssl rand -hex 32)"
        echo "Filled $key"
    fi
done

if has_key CREDENTIAL_ENCRYPTION_KEY && [ -z "$(current CREDENTIAL_ENCRYPTION_KEY)" ]; then
    set_value CREDENTIAL_ENCRYPTION_KEY "$(openssl rand -base64 32)"
    echo "Filled CREDENTIAL_ENCRYPTION_KEY"
fi

echo "Done. Still to fill by hand: the Clerk, Cloudflare and Alexa values (docs/human-setup.md)."
