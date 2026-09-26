# Backend Agent Guide

## Stack
Python 3.12 · FastAPI · SQLAlchemy (async) · Alembic · PydanticAI · PostgreSQL · Clerk JWT · Playwright · curl_cffi

## Structure
```
app/
├── main.py              # App factory, router registration, middleware
├── config.py            # Settings via pydantic-settings (reads .env)
├── routes/              # One router per domain — import in main.py
│   ├── voice.py         # POST /api/voice-requests  (webhook — no JWT, uses shared secret)
│   ├── grocery.py       # /api/grocery-items
│   ├── orders.py        # /api/orders
│   ├── analysis.py      # /api/analysis
│   ├── settings.py      # /api/settings/*
│   ├── logs.py          # /api/logs
│   └── ws.py            # WebSocket /ws
├── auth/
│   ├── provider.py      # AuthProvider protocol — swap implementations here
│   └── clerk.py         # Clerk JWT verification (current implementation)
├── models/
│   ├── db.py            # SQLAlchemy ORM models
│   └── schemas.py       # Pydantic request/response schemas
├── services/            # Business logic — no HTTP, no direct DB sessions
│   ├── optimiser.py     # Order cost optimisation algorithm
│   ├── encryption.py    # AES-256 encrypt/decrypt
│   └── stores/
│       ├── base.py      # StoreClient protocol
│       ├── woolworths.py
│       └── coles.py
└── db/
    ├── session.py       # Async session factory + get_db dependency
    └── base.py          # SQLAlchemy Base
```

## Route pattern
```python
# app/routes/grocery.py
from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.provider import get_current_user, AuthUser
from app.db.session import get_db
from app.models.schemas import GroceryItemResponse, NewGroceryItem
from app.services.grocery import GroceryService

router = APIRouter(prefix="/api/grocery-items", tags=["grocery"])

@router.get("/", response_model=list[GroceryItemResponse])
async def list_items(
    user: AuthUser = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> list[GroceryItemResponse]:
    return await GroceryService(db).list_active()

@router.post("/", response_model=GroceryItemResponse, status_code=201)
async def add_item(
    body: NewGroceryItem,
    user: AuthUser = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> GroceryItemResponse:
    return await GroceryService(db).add(body, added_by=user.id)
```

## Auth
```python
# All routes except /api/voice-requests use JWT
user: AuthUser = Depends(get_current_user)

# Voice webhook uses shared secret
Depends(verify_webhook_secret)
```

All auth logic goes through `app/auth/provider.py` — never call Clerk SDK directly in a route.

## Service pattern
```python
# app/services/grocery.py
class GroceryService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def list_active(self) -> list[GroceryItemResponse]:
        result = await self.db.execute(
            select(GroceryItem).where(GroceryItem.status == "active")
        )
        return [GroceryItemResponse.model_validate(row) for row in result.scalars()]
```

Services receive a `db` session — they do not create one. No HTTP calls in services (that's `stores/`).

## WebSocket (real-time push)
```python
# app/routes/ws.py
# ConnectionManager tracks open sessions
# Broadcasts: {"type": "voice_request_added", "count": n}
# Fired by: voice route after inserting a new voice_request
```

## File length
Hard limit: 300 lines. Extract service logic to `services/` first. If service is still over, split by responsibility into submodules.

## Credentials (AES-256)
```python
from app.services.encryption import encrypt, decrypt

# Store
encrypted = encrypt(plaintext, key=settings.CREDENTIAL_ENCRYPTION_KEY)
# Retrieve
plaintext = decrypt(encrypted, key=settings.CREDENTIAL_ENCRYPTION_KEY)
```

Key comes from env var — never hardcoded, never logged, never returned to frontend.

## Skills in this directory
- `skills/store-integration.md` — store APIs, Playwright, Akamai mitigations
