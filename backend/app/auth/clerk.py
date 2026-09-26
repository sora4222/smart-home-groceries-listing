"""Clerk JWT verification — the current `AuthProvider` implementation.

Verifies the bearer token from the frontend against Clerk's published JWKS
(fetched once and cached for the process lifetime). Swap this file for a
different provider without touching `app/auth/provider.py`'s callers.
"""

import time

import httpx
from fastapi import HTTPException, status
from jose import jwt
from jose.exceptions import JOSEError

from app.auth.provider import AuthUser
from app.config import get_settings

_JWKS_CACHE_TTL_SECONDS = 3600
_jwks_cache: dict | None = None
_jwks_fetched_at: float = 0.0


async def _get_jwks() -> dict:
    global _jwks_cache, _jwks_fetched_at
    settings = get_settings()
    if not settings.clerk_jwks_url:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="CLERK_JWKS_URL is not configured on the server",
        )
    now = time.monotonic()
    if _jwks_cache is None or (now - _jwks_fetched_at) > _JWKS_CACHE_TTL_SECONDS:
        async with httpx.AsyncClient(timeout=5.0) as client:
            resp = await client.get(settings.clerk_jwks_url)
            resp.raise_for_status()
            _jwks_cache = resp.json()
            _jwks_fetched_at = now
    return _jwks_cache


class ClerkAuthProvider:
    """Verifies Clerk-issued session JWTs via JWKS."""

    async def verify_token(self, authorization: str | None) -> AuthUser:
        if not authorization or not authorization.startswith("Bearer "):
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Missing bearer token",
            )
        token = authorization.removeprefix("Bearer ").strip()
        jwks = await _get_jwks()
        try:
            header = jwt.get_unverified_header(token)
            key = next(
                (k for k in jwks.get("keys", []) if k.get("kid") == header.get("kid")),
                None,
            )
            if key is None:
                raise JOSEError("No matching JWKS key for token")
            claims = jwt.decode(
                token,
                key,
                algorithms=[header.get("alg", "RS256")],
                options={"verify_aud": False},
            )
        except JOSEError as exc:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail=f"Invalid session token: {exc}",
            ) from exc

        user_id = claims.get("sub")
        if not user_id:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Token missing subject claim",
            )
        return AuthUser(id=user_id, email=claims.get("email"))
