# Contributing

This monorepo contains the Rust/Axum API, React/TanStack Start frontend, Flask
Alexa verification sidecar and Tauri desktop client. Use pnpm for TypeScript,
cargo for Rust and uv for the Python sidecar. See [AGENTS.md](AGENTS.md) for code
ownership, single responsibility, file limits and tests alongside changes.

## Software Lifecycle

1. Install the toolchains and native dependencies in [production setup](docs/production.md),
   plus Docker for development/tests. Run `make setup`, then `make setup-env`.
   Complete [human provisioning](docs/human-setup.md) for the selected integrations.
2. Run `make dev` for the complete watched stack. Ctrl-C or `make dev-stop` removes
   only this worktree's containers and network while retaining its volumes.
   The desktop client has a separate `make hm-dev-desktop` workflow.
3. Run focused `test-<component>` commands as you work. `make check` applies
   formatting/lint fixes, checks types and file lengths, and runs all tests.
   Use `human=1` for full logs. Inspect formatter/linter changes before committing.
4. Add forward-only SQLx migrations with `make migration-new name="description"`.
   `make update-db` applies them to the selected database and retains records.
5. `make build` assembles server components into `build/production` from
   `build/backend`, `build/frontend` and `build/alexa`. `make build-desktop`
   separately packages the native client into `build/desktop`. Each replaces
   previous output only after its build succeeds.
6. Install on the server with `sudo make install-server`. Use
   `make hm-update-server` for the pull, build, migration check, backup and install
   workflow. Missing system dependencies require separate installation or an
   explicit `ALLOW_DEPENDENCY_INSTALL=1`. See [production operations](docs/production.md).

Keep credentials in ignored configuration files or 1Password. Never commit
secret values. Use small commits with a conventional prefix such as `feat:`
or `fix:` and stage only files belonging to that change.
