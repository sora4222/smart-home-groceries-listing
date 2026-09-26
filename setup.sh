#!/usr/bin/env bash
# Google Home Grocery List — repo scaffold
# Usage: bash setup.sh [project-dir]
set -e

PROJECT=${1:-google-home-grocery}
echo "→ Scaffolding $PROJECT/"
mkdir -p "$PROJECT" && cd "$PROJECT"

# ── Git ───────────────────────────────────────────────────────────
git init -q
cat > .gitignore << 'EOF'
.env
__pycache__/
*.pyc
.venv/
node_modules/
dist/
.DS_Store
*.log
.playwright/
EOF

# ── Root files ────────────────────────────────────────────────────
touch README.md
touch FEATURE_VOICE.md FEATURE_GROCERY_LIST.md FEATURE_STORE_INTEGRATION.md \
      FEATURE_ORDER_OPTIMISATION.md FEATURE_CHECKOUT.md FEATURE_SPENDING_ANALYSIS.md

cat > .env.example << 'EOF'
# PostgreSQL
POSTGRES_DB=grocery
POSTGRES_USER=grocery
POSTGRES_PASSWORD=changeme
DATABASE_URL=postgresql+asyncpg://grocery:changeme@db:5432/grocery

# Clerk
CLERK_SECRET_KEY=
CLERK_PUBLISHABLE_KEY=

# Google Home webhook shared secret (generate: openssl rand -hex 32)
VOICE_WEBHOOK_SECRET=

# Cloudflare Tunnel
CLOUDFLARE_TUNNEL_TOKEN=

# AES-256 credential encryption key (generate: openssl rand -base64 32)
CREDENTIAL_ENCRYPTION_KEY=
EOF

cat > docker-compose.yml << 'EOF'
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-grocery}
      POSTGRES_USER: ${POSTGRES_USER:-grocery}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pg_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-grocery}"]
      interval: 5s
      retries: 5

  backend:
    build: ./backend
    depends_on:
      db:
        condition: service_healthy
    env_file: .env
    volumes:
      - ./backend:/app
    ports:
      - "8000:8000"
    command: uv run uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload

  cloudflared:
    image: cloudflare/cloudflared:latest
    restart: unless-stopped
    command: tunnel --no-autoupdate run
    environment:
      TUNNEL_TOKEN: ${CLOUDFLARE_TUNNEL_TOKEN}

volumes:
  pg_data:
EOF

cat > Makefile << 'EOF'
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
EOF

# ── Backend ───────────────────────────────────────────────────────
echo "→ Setting up backend/"
mkdir -p backend
cd backend

uv init --python ">=3.12" -q
uv add \
  fastapi "uvicorn[standard]" \
  "sqlalchemy[asyncio]" "psycopg[binary]" alembic \
  pydantic "pydantic-settings" python-jose cryptography \
  websockets "curl-cffi" playwright -q

uv add --dev \
  pytest pytest-asyncio httpx \
  "testcontainers[postgres]" ruff ty -q

# Directory structure
mkdir -p \
  app/routes app/auth app/models app/services/stores app/db \
  alembic/versions \
  tests/unit tests/integration \
  skills

# Stub files
touch app/__init__.py app/main.py app/config.py
touch app/auth/__init__.py app/auth/provider.py app/auth/clerk.py
touch app/models/__init__.py app/models/db.py app/models/schemas.py
touch app/services/__init__.py \
      app/services/optimiser.py \
      app/services/encryption.py \
      app/services/stores/__init__.py \
      app/services/stores/woolworths.py \
      app/services/stores/coles.py \
      app/services/stores/base.py
touch app/db/__init__.py app/db/session.py app/db/base.py
touch app/routes/__init__.py \
      app/routes/voice.py \
      app/routes/grocery.py \
      app/routes/orders.py \
      app/routes/analysis.py \
      app/routes/settings.py \
      app/routes/logs.py \
      app/routes/ws.py
touch tests/__init__.py tests/unit/__init__.py tests/integration/__init__.py
touch tests/conftest.py

# Alembic
uv run alembic init alembic -q

# Playwright browsers
uv run playwright install chromium -q

touch AGENT.md skills/store-integration.md

cat > Dockerfile << 'EOF'
FROM python:3.12-slim
WORKDIR /app
RUN pip install uv -q
COPY pyproject.toml uv.lock* ./
RUN uv sync --frozen -q
COPY . .
EOF

cd ..

# ── Frontend ──────────────────────────────────────────────────────
echo "→ Setting up frontend/"
mkdir -p frontend
cd frontend

pnpm create vite . --template react-ts --yes -q

pnpm add \
  "@tanstack/react-router" \
  "@clerk/clerk-react" \
  sonner -q

pnpm add -D \
  tailwindcss "@tailwindcss/vite" \
  prettier eslint \
  "@typescript-eslint/eslint-plugin" \
  "@typescript-eslint/parser" \
  "eslint-config-prettier" \
  vitest "@vitest/coverage-v8" \
  "@testing-library/react" "@testing-library/jest-dom" \
  "@playwright/test" -q

# Shadcn/ui
pnpm dlx shadcn@latest init --yes -q

# Route structure
mkdir -p \
  src/routes/settings \
  src/components/ui \
  src/components/grocery \
  src/components/pending \
  src/components/order \
  src/components/analysis \
  src/components/settings \
  src/lib src/hooks \
  e2e skills

touch src/routes/__root.tsx \
      src/routes/index.tsx \
      src/routes/pending.tsx \
      src/routes/order.tsx \
      src/routes/analysis.tsx \
      src/routes/logs.tsx \
      src/routes/settings/index.tsx \
      src/routes/settings/item-rules.tsx \
      src/routes/settings/stores.tsx \
      src/routes/settings/delivery.tsx

touch src/lib/api.ts src/lib/ws.ts
touch AGENT.md skills/routing.md skills/components.md

cd ..

# ── Root skills ───────────────────────────────────────────────────
mkdir -p skills
touch skills/testing.md skills/git-commits.md skills/makefile.md skills/migrations.md

# ── Done ─────────────────────────────────────────────────────────
echo ""
echo "✓ Done. Next steps:"
echo "  1. cp .env.example .env  →  fill in secrets"
echo "  2. Copy AGENT.md and skills/ files from repo into place"
echo "  3. make up"
echo "  4. make migrate"
echo "  5. Frontend: http://localhost:5173"
echo "     API docs: http://localhost:8000/docs"
