#!/bin/sh
# First step of `make hm-update-server`, as your own user: bring the checked-out branch
# up to date with its upstream, without merging or discarding anything.
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
die() { echo "make hm-update-server: $*" >&2; exit 1; }

[ "$(id -u)" -ne 0 ] || die "run as your own user, not with sudo; it asks for sudo when it installs"
branch=$(git -C "$root" symbolic-ref --quiet --short HEAD) || die "no branch is checked out (detached HEAD)"
if [ -n "$(git -C "$root" status --porcelain --untracked-files=no)" ]; then
  die "the checkout has uncommitted changes; commit or stash them, so what is deployed is a commit"
fi

if git -C "$root" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' > /dev/null 2>&1; then
  echo "==> git pull --ff-only ($branch)"
  git -C "$root" pull --ff-only --quiet || die "$branch cannot fast-forward to its upstream; resolve it by hand"
else
  echo "==> $branch has no upstream; deploying it as it is"
fi
echo "==> deploying $branch at $(git -C "$root" log -1 --format='%h %s')"
