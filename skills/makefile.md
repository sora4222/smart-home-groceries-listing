# Makefile

## Target groups
| Group | Targets |
|---|---|
| Lifecycle | `up`, `down`, `restart`, `alexa-up` |
| Test | `test`, `test-backend`, `test-frontend`, `test-alexa`, `hm-test*` |
| DB | `migrate`, `migration-new name="<desc>"` |
| Lint | `lint`, `lint-fix`, `check` |
| Build | `build` |

## Adding a target
```makefile
## Group heading (two hashes)

# Standard target — compact output for agent use
new-target:
	@cd backend && cargo some-command --quiet 2>&1 | grep -E "^error|test result" || true

# Human-readable variant — full output
hm-new-target:
	cd backend && cargo some-command
```

## Output rules
- Non-`hm-*` targets: suppress everything but errors, warnings and test
  summaries. Prefix with `@`. Pipe through `grep -E "..."`.
- `hm-*` targets: full output, no `@`, no grep filter.
- A target that adds a new service must also be reflected in `up`/`down`.
- `|| true` after a filtered pipeline, so a `grep` that matches nothing does
  not fail the target.

## Running commands per language
```makefile
# Backend (Rust) — runs on the host, not in the container. Cargo's
# incremental cache lives in backend/target and would be lost each time.
cd backend && cargo <command>

# Alexa bridge sidecar (Python)
cd sidecars/alexa-bridge && uv run <command>

# Frontend (Node)
cd frontend && pnpm <command>

# DB (psql)
docker compose exec db psql -U grocery -d grocery -c "<sql>"

# The backend binary inside its container
docker compose exec backend grocery-backend --migrate-only
```

Note the difference from the previous Python setup: backend commands no longer
go through `docker compose exec`. The tests need only a reachable PostgreSQL,
not a Docker daemon, so running them on the host is both faster and what CI
does.

## Common patterns
```makefile
# Run with an argument
migration-new:
	@cd backend/migrations && touch "$$(next_version)_$(name).sql"
# Usage: make migration-new name="add item rules table"

# A target that needs the database up first
seed: up
	@cd backend && cargo run --quiet --bin seed 2>&1 | tail -3
```

## Multi-agent worktrees
Concurrent agents share one machine, so anything bound to a fixed port or a
fixed database name will collide. Override per worktree:

```bash
BIND_ADDRESS=127.0.0.1:8001 \
DATABASE_URL=postgres://grocery:pw@localhost:5432/grocery_wt2 \
  make test-backend
```

`#[sqlx::test]` already isolates each test in its own throwaway database, so
two agents running `make test-backend` against the same server are safe. It is
`make up` and a long-running `cargo run` that need distinct ports.
