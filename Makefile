.PHONY: up down restart test test-backend test-frontend migrate lint \
        lint-fix build hm-up hm-test hm-test-backend hm-test-frontend

## Lifecycle
up:
	@docker compose up -d 2>&1 | grep -E "error|Error|started|healthy" || true

down:
	@docker compose down -q

restart: down up

## Tests (compact — for agent use)
test: test-backend test-frontend

test-backend:
	@docker compose exec backend uv run pytest -q 2>&1 | tail -5

test-frontend:
	@cd frontend && pnpm test:run --reporter=dot 2>&1 | tail -5

## Database
migrate:
	@docker compose exec backend uv run alembic upgrade head -q

migration-new:
	docker compose exec backend uv run alembic revision --autogenerate -m "$(name)"

## Lint
lint:
	@docker compose exec backend uv run ruff check . -q 2>&1 | grep -E "error|^$$" || true
	@docker compose exec backend uv run ty check . 2>&1 | grep -E "error|^$$" || true
	@cd frontend && pnpm lint --quiet 2>&1 | grep -E "error|^$$" || true

lint-fix:
	@docker compose exec backend uv run ruff check --fix . -q
	@cd frontend && pnpm lint --fix --quiet

## Build
build:
	@docker compose build -q

## Human-readable variants (hm-*)
hm-up:
	docker compose up

hm-test: hm-test-backend hm-test-frontend

hm-test-backend:
	docker compose exec backend uv run pytest -v

hm-test-frontend:
	cd frontend && pnpm test:run
