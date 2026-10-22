---
name: makefile
description: Add or change this repository's Makefile targets, including compact agent output, human-readable variants, and worktree-safe execution.
---

# Make targets

Use canonical `dev`, `stop-all`, `check`, `test`, and `build`, without aliases.
Keep focused suites under `test-<component>`; browser coverage is
`test-frontend-playwright`. Human-only operations use `hm-`, including
`hm-dev-frontend`, `hm-compose` diagnostics and production `hm-*-server` targets. Update every caller when
renaming a command. `format` applies formatting; `check` verifies all gates. Preserve the existing
500-line script gate; authoring instructions can be stricter.
Preserve nonzero statuses, signal statuses and the structured test verdict policy.

Run Rust in `backend/` with cargo, Alexa in `sidecars/alexa-bridge/` with uv,
and frontend commands in `frontend/` with pnpm. Tests own disposable databases
and random ports; application configuration and a household stack are never test
infrastructure. See the testing skill for runner contracts.

`dev` uses `scripts/dev.mjs`, the base Compose file and `docker-compose.dev.yml`.
It supplies `SERVER_API_BASE_URL=http://backend:8000` for SSR while browsers
use the published API port. It waits for health, watches frontend source and rebuilds backend/bridge images.
Its finally block runs shutdown even on failure or cancellation. `stop-all`
uses the same overlay and worktree context, needs no application secrets, and
retains volumes. `scripts/dev-context.mjs` derives defaults from the real checkout
path; exported or selected-file project names and ports override them. Verify
startup failure, child failure, interruption, repeated shutdown and isolation.

`hm-dev-frontend` retains host-only frontend development against a remote API.
It needs only frontend settings and no Docker/database. It runs attached to its
own terminal; Compose shutdown does not own that separately launched process.

The Compose runner resolves `op://` settings on startup, explicitly maps values,
and strips `OP_*` credentials before running Docker. Shutdown loads only project
and port configuration. Keep the selected `--env-file` consistent; never replace
explicit mappings with raw `env_file: .env`. Profiles follow the resolved
`DATABASE_URL` and preserve unrelated profile choices. See
`docs/features/FEATURE_CONFIGURATION.md`.

Production `build` and `hm-install-server`, `hm-update-server`,
`hm-uninstall-server`, `hm-status`, `hm-logs` retain systemd deployment.
Build as the developer into `build/production`; install as root. Updates check
migrations and back up before installation. Settings flow through
`deploy/bin/grocery-env` and `scripts/service-components.mjs`.
