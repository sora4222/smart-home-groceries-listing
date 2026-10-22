#!/bin/sh
# Keep the existing 500-line gate, its boundary and generated-file exclusions.
set -eu
root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
task_dir=$(mktemp -d)
trap 'rm -rf "$task_dir"' EXIT
for suffix in rs py ts tsx; do
  path=$task_dir/source.$suffix
  awk 'BEGIN { for (i=0; i<500; i++) print "line" }' > "$path"
  printf '%s\n' "$path" | sh "$root/scripts/check-file-length.sh"
  printf 'line\n' >> "$path"
  if printf '%s\n' "$path" | sh "$root/scripts/check-file-length.sh" > "$task_dir/result"; then
    echo "FAIL: 501-line $suffix file passed" >&2
    exit 1
  fi
  grep -q '501 lines (limit 500)' "$task_dir/result"
done
cp "$path" "$task_dir/routeTree.gen.ts"
printf '%s\n' "$task_dir/routeTree.gen.ts" "$task_dir/removed.rs" | sh "$root/scripts/check-file-length.sh"
echo 'file-length: boundaries and exclusions passed'
