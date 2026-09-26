"""FastAPI application factory: middleware and router registration.

Add a new domain by writing `app/routes/<domain>.py` (an `APIRouter`) and
registering it here — this file should stay a thin wiring layer.
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.config import get_settings
from app.routes import grocery, voice, ws

settings = get_settings()

app = FastAPI(title="Google Home Grocery List API")

app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origin_list,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(voice.router)
app.include_router(grocery.router)
app.include_router(ws.router)


@app.get("/api/health", tags=["health"])
async def health() -> dict[str, str]:
    return {"status": "ok"}
