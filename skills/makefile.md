# Makefile

## Target groups
| Group | Targets |
|---|---|
| Lifecycle | `up`, `down`, `restart` |
| Test | `test`, `test-backend`, `test-frontend`, `hm-test`, `hm-test-backend`, `hm-test-frontend` |
| DB | `migrate`, `migration-new name="<desc>"` |
| Lint | `lint`, `lint-fix` |
| Build | `build` |

## Adding a target
```makefile
## Group heading (two hashes, shows in make help)

# Standard target — compact output for agent use
new-target:
	@docker compose exec backend uv run some-command 2>&1 | grep -E "error|Error|passed|failed" || true

# Human-readable variant — full output
hm-new-target:
	docker compose exec backend uv run some-command
```

## Output rules
- Non-`hm-*` targets: suppress all output except errors, warnings, and test summary.
  Prefix commands with `@`. Pipe through `grep -E "..."` to filter.
- `hm-*` targets: full output, no `@`, no grep filter.
- A target that adds a new service must also be included in `up`/`down`.

## Running commands inside containers
```makefile
# Backend (Python)
docker compose exec backend uv run <command>

# Frontend (Node)
cd frontend && pnpm <command>

# DB (psql)
docker compose exec db psql -U grocery -d grocery -c "<sql>"
```

## Common patterns
```makefile
# Run with argument
migration-new:
	docker compose exec backend uv run alembic revision --autogenerate -m "$(name)"
# Usage: make migration-new name="add item rules table"

# Wait for service before acting
seed:
	@docker compose exec backend uv run python -m app.db.seed -q
```
