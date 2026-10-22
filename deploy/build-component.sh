#!/usr/bin/env bash
# Build one component into build/<component>, replacing output only after success.
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
component=${1:?usage: build-component.sh backend|frontend|alexa|desktop}
case $component in backend|frontend|alexa|desktop) ;; *) echo 'unknown build component' >&2; exit 1 ;; esac
[ "$(id -u)" -ne 0 ] || { echo 'build as your own user' >&2; exit 1; }
out=$root/build/$component
mkdir -p "$root/build"
stage=$(mktemp -d "$root/build/.$component.XXXXXX")
trap 'rm -rf "$stage"' EXIT
# Keep compiler caches inside this checkout by default, even with a shared shell setting.
case $component in
  backend)
    target=$root/backend/target
    CARGO_TARGET_DIR=$target cargo build --release --locked --quiet --manifest-path "$root/backend/Cargo.toml"
    install -m 755 "$target/release/grocery-backend" "$stage/grocery-backend"
    ;;
  frontend)
    pnpm --dir "$root/frontend" install --frozen-lockfile --silent
    node "$root/scripts/build-frontend.mjs"
    cp -a "$root/frontend/.output" "$stage/.output"
    ;;
  alexa)
    python=${GROCERY_PYTHON:-/usr/bin/python3}
    export UV_PYTHON_DOWNLOADS=never
    uv venv --quiet --relocatable --python "$python" "$stage/venv"
    UV_PROJECT_ENVIRONMENT=$stage/venv uv sync --quiet --frozen --no-dev \
      --no-editable --compile-bytecode --python "$python" --project "$root/sidecars/alexa-bridge"
    "$stage/venv/bin/python" -c 'import alexa_bridge.app, gunicorn'
    ;;
  desktop)
    target=$root/desktop/src-tauri/target
    CARGO_TARGET_DIR=$target pnpm --dir "$root/desktop" build
    cp -a "$target/release/bundle/." "$stage/"
    ;;
esac
rm -rf "$out"
mv "$stage" "$out"
trap - EXIT
printf 'built build/%s\n' "$component"
