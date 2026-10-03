---
name: makefile
description: Add or change this repository's Makefile targets, including compact agent output, human-readable variants, and worktree-safe execution.
---

# Make targets

Keep targets grouped under the existing headings. Ordinary targets are compact
for agents; an `hm-<target>` counterpart is appropriate when people need full
output. Preserve an existing target's output and exit-status behavior unless
changing it deliberately with matching tests.

Run each toolchain in its owned directory:

- Rust backend: `backend/` with `cargo` on the host.
- Alexa bridge: `sidecars/alexa-bridge/` with `uv run`.
- Frontend: `frontend/` with `pnpm`.

Use `TEST_DATABASE_URL` for backend tests. New long-running services or
fixed-port targets must accept worktree-specific configuration so concurrent
checkouts do not collide. Keep `up`/`down` aligned when a target manages a
service. Prefer existing targets over duplicating their behavior.
