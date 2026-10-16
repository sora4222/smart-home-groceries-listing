.PHONY: up down restart test test-backend test-frontend test-alexa test-tooling e2e \
        migrate migration-new check lint lint-fix build alexa-up setup-env \
        run-env hm-up hm-test hm-test-backend hm-test-frontend hm-test-alexa hm-test-tooling hm-e2e

# Tests provision their own Docker databases and random host ports.

# Any local command can use the same source and provider boundary.
run-env:
	@node scripts/run-env.mjs $(cmd)

## First-time setup
# Makes .env from .env.example and fills every random secret that is still
# empty. Never overwrites a value. See docs/human-setup.md, part 1.
setup-env:
	@sh scripts/fill-env.sh

## Lifecycle
# Database profiles follow the resolved URL, including older .env files.
up:
	@node scripts/compose.mjs up -d

# Shutdown selects the same project without resolving application secrets.
down:
	@node scripts/compose.mjs down

restart: down up

alexa-up:
	@node scripts/compose.mjs up -d alexa-bridge

## Tests (compact — for agent use)
test: test-backend test-frontend test-alexa test-tooling e2e

test-backend:
	@sh scripts/backend-tests.sh --compact

# Tests for the scripts the Makefile itself relies on.
test-tooling:
	@sh scripts/environment.test.sh
	@node --test scripts/environment-contract.test.mjs scripts/compose.test.mjs scripts/process.test.mjs scripts/test-results.test.mjs
	@sh scripts/test-database.test.sh
	@sh scripts/fill-env.test.sh
	@sh scripts/check-file-length.test.sh
	@sh scripts/check-migrations.test.sh
	@sh scripts/database-profiles.test.sh

test-frontend:
	@node scripts/test-command.mjs vitest frontend pnpm test:run

test-alexa:
	@node scripts/test-command.mjs pytest sidecars/alexa-bridge uv run pytest -q --disable-warnings

# Owns its database, backend, and frontend; never uses a running household stack.
e2e:
	@node scripts/e2e-tests.mjs

## Database
# Migrations are embedded in the binary and applied at startup, so the running
# backend is already migrated. This target is for applying them without
# starting the server.
migrate:
	@cd backend && cargo run --quiet --bin grocery-backend -- --migrate-only 2>&1 | tail -3 || \
	 docker compose exec backend grocery-backend --migrate-only 2>&1 | tail -3

# sqlx names migrations <version>_<description>.sql. Numbering is sequential,
# so check the highest existing file before adding one.
migration-new:
	@cd backend/migrations && \
	 next=$$(printf "%04d" $$(( $$(ls -1 *.sql 2>/dev/null | sed 's/_.*//' | sort -n | tail -1 | sed 's/^0*//' | grep . || echo 0) + 1 ))) && \
	 slug=$$(echo "$(name)" | tr '[:upper:] ' '[:lower:]_') && \
	 touch "$${next}_$${slug}.sql" && \
	 echo "created backend/migrations/$${next}_$${slug}.sql"

## Lint
lint:
	@cd backend && cargo clippy --all-targets --quiet -- -D warnings
	@cd backend && cargo fmt --check
	@cd sidecars/alexa-bridge && uv run ruff check .
	@cd sidecars/alexa-bridge && uv run ruff format --check .
	@cd frontend && pnpm check
	@cd frontend && pnpm exec tsc --noEmit

lint-fix:
	@cd backend && cargo clippy --all-targets --fix --allow-dirty --quiet 2>&1 | tail -2
	@cd backend && cargo fmt
	@cd sidecars/alexa-bridge && uv run ruff check --fix . && uv run ruff format .
	@cd frontend && pnpm check --write

## Build
build:
	@docker compose build -q

check: lint
	@cd backend && cargo check --all-targets --quiet
	@git ls-files backend frontend/src sidecars | sh scripts/check-file-length.sh

## Human-readable variants (hm-*)
hm-up:
	@node scripts/compose.mjs up

hm-test: hm-test-backend hm-test-frontend hm-test-alexa hm-test-tooling hm-e2e

hm-test-backend:
	@sh scripts/backend-tests.sh

hm-test-tooling:
	@$(MAKE) test-tooling

hm-test-frontend:
	node scripts/test-command.mjs vitest frontend pnpm test:run

hm-test-alexa:
	node scripts/test-command.mjs pytest sidecars/alexa-bridge uv run pytest -v

hm-e2e:
	@node scripts/e2e-tests.mjs
