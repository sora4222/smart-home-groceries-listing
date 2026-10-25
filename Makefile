# Predictable lifecycle commands. Use human=1 or the legacy hm- aliases for full logs.
.DEFAULT_GOAL := check
HUMAN = $(if $(filter 1 true,$(human)),--human,)
RUN = node scripts/command.mjs $(HUMAN) $@ --
COMPONENTS := backend frontend alexa desktop
.PHONY: all setup setup-env run-env dev dev-stop stop-all hm-compose hm-dev-frontend \
        test test-tooling test-frontend-playwright update-db reset-db migration-new \
        check check-format check-file-length check-desktop format lint hm-fix \
        type-backend type-frontend type-desktop build install-server uninstall-server \
        hm-install-server hm-uninstall-server hm-update-server hm-status hm-logs \
        hm-dev hm-test hm-test-tooling hm-test-frontend-playwright hm-dev-desktop \
        create-application-desktop $(addprefix test-,$(COMPONENTS)) \
        $(addprefix hm-test-,$(COMPONENTS)) $(addprefix build-,$(COMPONENTS)) \
        $(addprefix format-,$(COMPONENTS)) $(addprefix lint-,$(COMPONENTS))

all: check
	@$(MAKE) --no-print-directory build

# Any local command can use the shared configuration and provider boundary.
run-env:
	@node scripts/run-env.mjs $(cmd)

## Setup. Never replace an existing environment file or its database credentials.
setup:
	@$(RUN) pnpm install --frozen-lockfile --silent
	@$(RUN) pnpm --dir frontend install --frozen-lockfile --silent
	@cd frontend && node ../scripts/command.mjs $(HUMAN) $@ -- pnpm exec playwright install chromium
	@cd sidecars/alexa-bridge && node ../../scripts/command.mjs $(HUMAN) $@ -- uv sync --frozen
	@cd backend && node ../scripts/command.mjs $(HUMAN) $@ -- cargo fetch --locked --quiet
	@$(RUN) pnpm --dir desktop install --frozen-lockfile --silent
	@cd desktop/src-tauri && node ../../scripts/command.mjs $(HUMAN) $@ -- cargo fetch --locked --quiet

setup-env:
	@sh scripts/fill-env.sh

## Development. Context and cleanup are scoped to this worktree.
dev:
	@node scripts/dev.mjs $(HUMAN)

dev-stop:
	@node scripts/dev-compose.mjs down

stop-all: dev-stop

hm-compose:
	@node scripts/dev-compose.mjs $(args)

# Frontend only, against the configured remote backend; no local database.
WEB_PORT ?= 3000
hm-dev-frontend:
	@pnpm --dir frontend install --frozen-lockfile --silent
	@cd frontend && pnpm exec vite dev --port $(WEB_PORT) --strictPort

## Tests. Database runners own disposable containers on random ports.
test: $(addprefix test-,$(COMPONENTS)) test-tooling test-frontend-playwright

test-backend:
	@sh scripts/backend-tests.sh $(if $(HUMAN),,--compact)

test-tooling:
	@$(RUN) sh scripts/tooling-tests.sh

test-frontend:
	@cd frontend && node ../scripts/command.mjs $(HUMAN) $@ -- pnpm test:run

test-alexa:
	@cd sidecars/alexa-bridge && node ../../scripts/command.mjs $(HUMAN) $@ -- uv run pytest -q --disable-warnings

test-frontend-playwright:
	@node scripts/e2e-tests.mjs $(HUMAN)

test-desktop:
	@node scripts/desktop-tests.mjs $(HUMAN)

## Database. Forward-only SQLx migrations, also applied at backend startup.
update-db:
	@node scripts/run-env.mjs cargo run --quiet --manifest-path backend/Cargo.toml --bin grocery-backend -- --migrate-only

# Deliberately explicit and limited to the owned Compose database, never DATABASE_URL.
reset-db:
	@node scripts/reset-db.mjs $(if $(filter reset,$(confirm)),--confirm,)

migration-new:
	@cd backend/migrations && \
	 next=$$(printf "%04d" $$(( $$(ls -1 *.sql 2>/dev/null | sed 's/_.*//' | sort -n | tail -1 | sed 's/^0*//' | grep . || echo 0) + 1 ))) && \
	 slug=$$(echo "$(name)" | tr '[:upper:] ' '[:lower:]_') && \
	 touch "$${next}_$${slug}.sql" && \
	 echo "created backend/migrations/$${next}_$${slug}.sql"

## Checks. Independent components share make -j jobs within each phase.
# Finish source rewrites before readers, and finish checks before all builds.
check: format
	@$(MAKE) --no-print-directory lint
	@$(MAKE) --no-print-directory type-backend type-frontend type-desktop check-file-length test

format: $(addprefix format-,$(COMPONENTS))
lint: $(addprefix lint-,$(COMPONENTS))

format-backend:
	@cd backend && node ../scripts/command.mjs $(HUMAN) $@ -- cargo fmt
format-frontend:
	@cd frontend && node ../scripts/command.mjs $(HUMAN) $@ -- pnpm exec biome format --write .
format-alexa:
	@cd sidecars/alexa-bridge && node ../../scripts/command.mjs $(HUMAN) $@ -- uv run ruff format .
format-desktop:
	@cd desktop/src-tauri && node ../../scripts/command.mjs $(HUMAN) $@ -- cargo fmt

lint-backend:
	@cd backend && node ../scripts/command.mjs $(HUMAN) $@ -- cargo clippy --all-targets --fix --allow-dirty --quiet -- -D warnings
lint-frontend:
	@cd frontend && node ../scripts/command.mjs $(HUMAN) $@ -- pnpm check --write
lint-alexa:
	@cd sidecars/alexa-bridge && node ../../scripts/command.mjs $(HUMAN) $@ -- uv run ruff check --fix .
lint-desktop:
	@cd desktop/src-tauri && node ../../scripts/command.mjs $(HUMAN) $@ -- cargo clippy --locked --all-targets --fix --allow-dirty --quiet -- -D warnings

type-backend:
	@cd backend && node ../scripts/command.mjs $(HUMAN) $@ -- cargo check --all-targets --quiet
type-frontend:
	@cd frontend && node ../scripts/command.mjs $(HUMAN) $@ -- pnpm exec tsc --noEmit
type-desktop:
	@cd desktop/src-tauri && node ../../scripts/command.mjs $(HUMAN) $@ -- cargo check --locked --all-targets --quiet

# Read-only formatting gate retained for CI and focused checks.
check-format:
	@cd backend && cargo fmt --check
	@cd desktop/src-tauri && cargo fmt --check
	@cd sidecars/alexa-bridge && uv run ruff format --check .
	@cd frontend && pnpm exec biome format .

check-file-length:
	@git ls-files backend frontend/src sidecars desktop/src-tauri/src | sh scripts/check-file-length.sh

check-desktop: format-desktop
	@$(MAKE) --no-print-directory lint-desktop type-desktop

hm-fix: format
	@$(MAKE) --no-print-directory lint

## Builds. Individual outputs in build/<component>; installable server in build/production.
build: build-backend build-frontend build-alexa
	@$(RUN) bash deploy/build.sh --assemble

build-backend build-frontend build-alexa:
	@$(RUN) bash deploy/build-component.sh $(@:build-%=%)

build-desktop:
	@$(RUN) bash deploy/build-component.sh desktop

create-application-desktop: build-desktop

## Production server. Explicit opt-in permits missing system dependency installation.
install-server:
	@bash deploy/start.sh $(args)

uninstall-server:
	@bash deploy/uninstall.sh

hm-install-server: install-server
hm-uninstall-server: uninstall-server

hm-update-server:
	@sudo /usr/local/sbin/grocery-update $(release)

package-server: build
	@node deploy/service/package.mjs

hm-status:
	@systemctl --no-pager status grocery.target 'grocery-*.service' ollama.service 2>/dev/null | grep -E '^(●|○|×)|Active:' || true

hm-logs:
	@journalctl --no-pager -n 100 -u 'grocery-*'

## Human output aliases share the canonical recipes and verdict policy.
hm-dev hm-test hm-test-tooling hm-test-frontend-playwright $(addprefix hm-test-,$(COMPONENTS)): human=1
hm-dev: dev
hm-test: test
hm-test-tooling: test-tooling
hm-test-frontend-playwright: test-frontend-playwright
$(addprefix hm-test-,$(COMPONENTS)): hm-test-%: test-%

hm-dev-desktop:
	@cd desktop && pnpm dev
