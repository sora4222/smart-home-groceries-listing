.PHONY: up down restart test test-backend test-frontend test-alexa test-tooling e2e \
        migrate migration-new lint lint-fix build alexa-up setup-env \
        hm-up hm-test hm-test-backend hm-test-frontend hm-test-alexa hm-test-tooling hm-e2e

# Where the backend's tests create their throwaway databases. Overridable per
# git worktree so concurrent agents do not collide. The default is built from
# POSTGRES_* in the environment or .env, aimed at the port Compose publishes:
# .env's own DATABASE_URL names the host `db`, which only resolves in Docker.
TEST_DATABASE_URL ?= $(shell sh scripts/test-database-url.sh)

## First-time setup
# Makes .env from .env.example and fills every random secret that is still
# empty. Never overwrites a value. See docs/human-setup.md, part 1.
setup-env:
	@sh scripts/fill-env.sh

## Lifecycle
up:
	@docker compose up -d 2>&1 | grep -E "error|Error|started|healthy" || true

# `docker compose down` has no quiet flag, so only errors are shown.
down:
	@docker compose down 2>&1 | grep -E "error|Error" || true

restart: down up

alexa-up:
	@docker compose up -d alexa-bridge 2>&1 | grep -E "error|Error|started" || true

## Tests (compact — for agent use)
test: up test-backend test-frontend test-alexa test-tooling

test-backend:
	@cd backend && DATABASE_URL='$(TEST_DATABASE_URL)' cargo test --quiet 2>&1 | grep -E "error|warning: unused|test result|FAILED|panicked" || true

# Tests for the scripts the Makefile itself relies on.
test-tooling:
	@sh scripts/test-database-url.test.sh | grep -E "FAIL|expected|actual" || echo "tooling: ok"
	@sh scripts/fill-env.test.sh | grep -E "FAIL|expected|actual" || echo "fill-env: ok"
	@sh scripts/check-file-length.test.sh | grep -E "FAIL|expected|actual" || echo "check-file-length: ok"
	@sh scripts/check-migrations.test.sh | grep -E "FAIL|expected|actual" || echo "check-migrations: ok"
	@sh scripts/check-env-files.test.sh | grep -E "FAIL|expected|actual" || echo "check-env-files: ok"

test-frontend:
	@cd frontend && pnpm test:run --reporter=dot 2>&1 | tail -5

test-alexa:
	@cd sidecars/alexa-bridge && uv run pytest -q 2>&1 | tail -5

# Not part of `make test`: this one drives a real browser against a running
# stack, so `make up` has to have happened first.
e2e:
	@cd frontend && pnpm e2e --reporter=line 2>&1 | tail -8

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
	@cd backend && cargo clippy --all-targets --quiet -- -D warnings 2>&1 | grep -E "^error|^warning" || true
	@cd backend && cargo fmt --check 2>&1 | grep -E "Diff in" || true
	@cd sidecars/alexa-bridge && uv run ruff check . 2>&1 | grep -vE "^All checks passed" || true
	@cd sidecars/alexa-bridge && uv run ruff format --check . 2>&1 | grep -E "Would reformat" || true
	@cd frontend && pnpm check 2>&1 | grep -E "error|^$$" || true
	@cd frontend && pnpm exec tsc --noEmit 2>&1 | grep -E "error" || true

lint-fix:
	@cd backend && cargo clippy --all-targets --fix --allow-dirty --quiet 2>&1 | tail -2
	@cd backend && cargo fmt
	@cd sidecars/alexa-bridge && uv run ruff check --fix . && uv run ruff format .
	@cd frontend && pnpm check --write

## Build
build:
	@docker compose build -q

check:
	@cd backend && cargo check --all-targets --quiet 2>&1 | grep -E "^error|^warning" || true

## Human-readable variants (hm-*)
hm-up:
	docker compose up

hm-test: hm-test-backend hm-test-frontend hm-test-alexa hm-test-tooling

hm-test-backend:
	cd backend && DATABASE_URL='$(TEST_DATABASE_URL)' cargo test

hm-test-tooling:
	sh scripts/test-database-url.test.sh

hm-test-frontend:
	cd frontend && pnpm test:run

hm-test-alexa:
	cd sidecars/alexa-bridge && uv run pytest -v

hm-e2e:
	cd frontend && pnpm e2e
