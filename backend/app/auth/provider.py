"""Auth abstraction.

Every route depends on `get_current_user` or `verify_webhook_secret` from
this module — never on a specific provider SDK. To substitute a different
auth provider (Auth0, Supabase Auth, ...), implement `AuthProvider` and
change `_provider` below; nothing else in the app needs to change.
"""

from dataclasses import dataclass
from typing import Protocol

from fastapi import Depends, Header, HTTPException, status

from app.config import get_settings


@dataclass(frozen=True)
class AuthUser:
    """The authenticated household member making the request."""

    id: str
    email: str | None = None


class AuthProvider(Protocol):
    """Contract any auth backend must satisfy."""

    async def verify_token(self, authorization: str | None) -> AuthUser:
        """Validate a bearer token and return the authenticated user, or raise."""
        ...


def _get_provider() -> AuthProvider:
    from app.auth.clerk import ClerkAuthProvider

    return ClerkAuthProvider()


async def get_current_user(
    authorization: str | None = Header(default=None),
) -> AuthUser:
    """FastAPI dependency: resolves the authenticated household member.

    All API routes except the Google Home webhook depend on this.
    """
    settings = get_settings()
    if settings.dev_auth_bypass:
        return AuthUser(id="dev-user", email="dev@example.local")

    provider = _get_provider()
    return await provider.verify_token(authorization)


async def verify_webhook_secret(
    x_webhook_secret: str | None = Header(default=None),
) -> None:
    """FastAPI dependency for the Google Home webhook (shared-secret auth).

    The webhook cannot carry a Clerk session, so it is authenticated with a
    secret shared out-of-band with the Cloudflare Tunnel / Google Action.
    """
    settings = get_settings()
    if not settings.voice_webhook_secret:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="VOICE_WEBHOOK_SECRET is not configured on the server",
        )
    if x_webhook_secret != settings.voice_webhook_secret:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid webhook secret",
        )


CurrentUser = Depends(get_current_user)
