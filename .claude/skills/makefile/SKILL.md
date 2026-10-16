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

Backend and browser test targets own disposable PostgreSQL Testcontainers.
Keep tests independent of application `.env`, profiles, secrets, and fixed
ports. Include browser tests in `test`; preserve command failures rather than
masking them with output filters. Keep process-group and container cleanup in
the owned runners. Prefer existing targets over duplicating their behavior.

Compose lifecycle commands use `scripts/compose.mjs` with the same selected
`--env-file`. Shutdown loads only Compose settings and preserves failures
without resolving application secrets. Startup resolves `op://` settings.
Keep authentication on the host and backend environment mappings explicit;
`env_file: .env` bypasses resolution. Test with fixture configuration through
`make test-tooling`, without displaying real resolved Compose configuration.
See `docs/features/FEATURE_CONFIGURATION.md` when adding settings.
Choose the database profile from the resolved `DATABASE_URL`: enable `db` for
the Compose hostname, remove it for external PostgreSQL, and preserve all
other profiles and the original configuration file.
