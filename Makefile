.PHONY: all dev stop-all setup setup-env run-env hm-dev-frontend hm-compose test test-backend \
        test-frontend test-alexa test-tooling test-frontend-playwright \
        update-db migration-new check check-format lint type-backend type-frontend \
        check-file-length format hm-fix build hm-install-server hm-update-server \
        hm-uninstall-server hm-status hm-logs hm-dev hm-test hm-test-backend \
        hm-test-frontend hm-test-alexa hm-test-tooling hm-test-frontend-playwright \
        hm-dev-desktop build-desktop test-desktop check-desktop

# Tests provision their own Docker databases and random host ports.

# Any local command can use the same source and provider boundary.
run-env:
	@node scripts/run-env.mjs $(cmd)

## First-time setup
setup:
	@pnpm install --frozen-lockfile --silent
	@cd frontend && pnpm install --frozen-lockfile --silent
	@cd frontend && pnpm exec playwright install chromium
	@cd sidecars/alexa-bridge && uv sync --frozen
	@cd backend && cargo fetch --locked --quiet
	@cd desktop && pnpm install --frozen-lockfile --silent

# Makes .env from .env.example and fills every random secret that is still
# empty. Never overwrites a value. See docs/human-setup.md, part 1.
setup-env:
	@sh scripts/fill-env.sh

## Lifecycle
.DEFAULT_GOAL := check

all: check test build

dev:
	@node scripts/dev.mjs

stop-all:
	@node scripts/dev-compose.mjs down

# Diagnostics and one-off service commands share the development project context.
hm-compose:
	@node scripts/dev-compose.mjs $(args)

# Only the web app, on this computer, talking to the backend at
# VITE_API_BASE_URL (for example a home server behind a Cloudflare Tunnel).
# No Docker, no database here. See docs/human-setup.md, "Optional: web app
# on your own computer". WEB_PORT must match an origin in the backend's
# CORS_ORIGINS.
WEB_PORT ?= 3000
hm-dev-frontend:
	@cd frontend && pnpm install --frozen-lockfile --silent
	@cd frontend && pnpm exec vite dev --port $(WEB_PORT) --strictPort

## Tests (compact — for agent use)
test: test-backend test-frontend test-alexa test-tooling test-frontend-playwright test-desktop

test-backend:
	@sh scripts/backend-tests.sh --compact

# Tests for the scripts the Makefile itself relies on.
test-tooling:
	@sh scripts/compose-config.test.sh
	@sh scripts/desktop.test.sh
	@sh scripts/environment.test.sh
	@node --test scripts/environment-contract.test.mjs scripts/compose.test.mjs scripts/dev.test.mjs scripts/dev-context.test.mjs scripts/process.test.mjs scripts/test-results.test.mjs
	@node --test scripts/service-env.test.mjs scripts/op-environment.test.mjs scripts/deploy-setup.test.mjs scripts/migration-plan.test.mjs scripts/build-frontend.test.mjs
	@sh scripts/grocery-env.test.sh
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
test-frontend-playwright:
	@node scripts/e2e-tests.mjs

## Database
# Migrations are embedded in the binary and applied at startup, so the running
# backend is already migrated. This target is for applying them without
# starting the server.
update-db:
	@node scripts/run-env.mjs cargo run --quiet --manifest-path backend/Cargo.toml --bin grocery-backend -- --migrate-only

# sqlx names migrations <version>_<description>.sql. Numbering is sequential,
# so check the highest existing file before adding one.
migration-new:
	@cd backend/migrations && \
	 next=$$(printf "%04d" $$(( $$(ls -1 *.sql 2>/dev/null | sed 's/_.*//' | sort -n | tail -1 | sed 's/^0*//' | grep . || echo 0) + 1 ))) && \
	 slug=$$(echo "$(name)" | tr '[:upper:] ' '[:lower:]_') && \
	 touch "$${next}_$${slug}.sql" && \
	 echo "created backend/migrations/$${next}_$${slug}.sql"

## Checks and formatting
check: check-format lint type-backend type-frontend check-file-length check-desktop

check-format:
	@cd backend && cargo fmt --check
	@cd sidecars/alexa-bridge && uv run ruff format --check .
	@cd frontend && pnpm exec biome format .

lint:
	@cd backend && cargo clippy --all-targets --quiet -- -D warnings
	@cd sidecars/alexa-bridge && uv run ruff check .
	@cd frontend && pnpm check

type-backend:
	@cd backend && cargo check --all-targets --quiet

type-frontend:
	@cd frontend && pnpm exec tsc --noEmit

check-file-length:
	@git ls-files backend frontend/src sidecars desktop/src-tauri/src | sh scripts/check-file-length.sh

format:
	@cd backend && cargo fmt
	@cd sidecars/alexa-bridge && uv run ruff format .
	@cd frontend && pnpm exec biome format --write .

hm-fix:
	@cd backend && cargo clippy --all-targets --fix --allow-dirty --quiet
	@cd sidecars/alexa-bridge && uv run ruff check --fix .
	@cd frontend && pnpm check --write
	@$(MAKE) --no-print-directory format

## Production server (no Docker) — docs/production.md
# Compiles every component into build/production as your own user.
build:
	@bash deploy/build.sh

# Run with sudo. Installs build/production to /opt/grocery as systemd
# services, plus PostgreSQL role, Ollama and cloudflared as configured.
hm-install-server:
	@bash deploy/install.sh

# Run as yourself (it asks for sudo). Deploys the checked-out branch:
# fast-forward pull, build, check pending migrations (stops on destructive
# ones unless ALLOW_DESTRUCTIVE=1), back up the database, install.
hm-update-server:
	@sh deploy/pull.sh
	@$(MAKE) --no-print-directory build
	@sudo env ALLOW_DESTRUCTIVE=$(ALLOW_DESTRUCTIVE) bash deploy/update.sh

hm-uninstall-server:
	@bash deploy/uninstall.sh

hm-status:
	@systemctl --no-pager status grocery.target 'grocery-*.service' ollama.service 2>/dev/null | grep -E '^(●|○|×)|Active:' || true

hm-logs:
	@journalctl --no-pager -n 100 -u 'grocery-*'

## Human-readable output
hm-dev:
	@node scripts/dev.mjs --human

hm-test: hm-test-backend hm-test-frontend hm-test-alexa hm-test-tooling hm-test-frontend-playwright test-desktop

hm-test-backend:
	@sh scripts/backend-tests.sh

hm-test-tooling:
	@$(MAKE) test-tooling

hm-test-frontend:
	node scripts/test-command.mjs vitest frontend pnpm test:run

hm-test-alexa:
	node scripts/test-command.mjs pytest sidecars/alexa-bridge uv run pytest -v

hm-test-frontend-playwright:
	@node scripts/e2e-tests.mjs

## Desktop client, no local server or database required
hm-dev-desktop:
	@cd desktop && pnpm dev

# Build only the native client. The public default URL is optional.
build-desktop:
	@cd desktop && pnpm build

test-desktop:
	@node scripts/desktop-tests.mjs

check-desktop:
	@cd desktop/src-tauri && cargo fmt --check
	@cd desktop/src-tauri && cargo clippy --locked --all-targets --quiet -- -D warnings
