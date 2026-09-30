.PHONY: up down restart test test-backend test-frontend test-alexa \
        migrate migration-new lint lint-fix build alexa-up \
        hm-up hm-test hm-test-backend hm-test-frontend hm-test-alexa

# Where the backend's tests create their throwaway databases. Overridable per
# git worktree so concurrent agents do not collide.
TEST_DATABASE_URL ?= postgres://grocery:$(POSTGRES_PASSWORD)@localhost:5432/postgres

## Lifecycle
up:
	@docker compose up -d 2>&1 | grep -E "error|Error|started|healthy" || true

down:
	@docker compose down -q

restart: down up

alexa-up:
	@docker compose up -d alexa-bridge 2>&1 | grep -E "error|Error|started" || true

## Tests (compact — for agent use)
test: test-backend test-frontend test-alexa

test-backend:
	@cd backend && cargo test --quiet 2>&1 | grep -E "error|warning: unused|test result|FAILED|panicked" || true

test-frontend:
	@cd frontend && pnpm test:run --reporter=dot 2>&1 | tail -5

test-alexa:
	@cd sidecars/alexa-bridge && uv run pytest -q 2>&1 | tail -5

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
	@cd frontend && pnpm lint --quiet 2>&1 | grep -E "error|^$$" || true

lint-fix:
	@cd backend && cargo clippy --all-targets --fix --allow-dirty --quiet 2>&1 | tail -2
	@cd backend && cargo fmt
	@cd sidecars/alexa-bridge && uv run ruff check --fix . && uv run ruff format .
	@cd frontend && pnpm lint --fix --quiet

## Build
build:
	@docker compose build -q

check:
	@cd backend && cargo check --all-targets --quiet 2>&1 | grep -E "^error|^warning" || true

## Human-readable variants (hm-*)
hm-up:
	docker compose up

hm-test: hm-test-backend hm-test-frontend hm-test-alexa

hm-test-backend:
	cd backend && cargo test

hm-test-frontend:
	cd frontend && pnpm test:run

hm-test-alexa:
	cd sidecars/alexa-bridge && uv run pytest -v
