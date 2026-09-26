"""Voice-request confirmation queue.

`POST /api/voice-requests` is the Google Home webhook (shared-secret auth,
called via the Cloudflare Tunnel — see docs/human-setup.md). Every other
route here requires a signed-in household member and drives the
"Pending Requests" screen in the web app.
"""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.provider import AuthUser, get_current_user, verify_webhook_secret
from app.db.session import get_db
from app.models.schemas import (
    DuplicateItemWarning,
    VoiceRequestAcceptResult,
    VoiceRequestCreate,
    VoiceRequestDecision,
    VoiceRequestResponse,
)
from app.services.voice import (
    DuplicateActiveItemError,
    VoiceRequestAlreadyDecided,
    VoiceRequestNotFound,
    VoiceService,
)

router = APIRouter(prefix="/api/voice-requests", tags=["voice"])


@router.post(
    "",
    response_model=VoiceRequestResponse,
    status_code=status.HTTP_201_CREATED,
    dependencies=[Depends(verify_webhook_secret)],
)
async def create_voice_request(
    body: VoiceRequestCreate,
    db: AsyncSession = Depends(get_db),
) -> VoiceRequestResponse:
    """Google Home Action webhook: 'Hey Google, add [quantity] [item] ...'."""
    service = VoiceService(db)
    request = await service.create_request(body)
    return VoiceRequestResponse.model_validate(request)


@router.get("", response_model=list[VoiceRequestResponse])
async def list_pending_voice_requests(
    db: AsyncSession = Depends(get_db),
    user: AuthUser = Depends(get_current_user),
) -> list[VoiceRequestResponse]:
    service = VoiceService(db)
    requests = await service.list_pending()
    return [VoiceRequestResponse.model_validate(r) for r in requests]


@router.post(
    "/{request_id}/accept",
    response_model=VoiceRequestAcceptResult,
    responses={409: {"model": DuplicateItemWarning}},
)
async def accept_voice_request(
    request_id: UUID,
    decision: VoiceRequestDecision | None = None,
    merge: bool = False,
    db: AsyncSession = Depends(get_db),
    user: AuthUser = Depends(get_current_user),
) -> VoiceRequestAcceptResult:
    service = VoiceService(db)
    try:
        request, item = await service.accept(
            request_id,
            decision or VoiceRequestDecision(),
            user_id=user.id,
            merge_with_duplicate=merge,
        )
    except VoiceRequestNotFound as exc:
        raise HTTPException(status.HTTP_404_NOT_FOUND, str(exc)) from exc
    except VoiceRequestAlreadyDecided as exc:
        raise HTTPException(status.HTTP_409_CONFLICT, str(exc)) from exc
    except DuplicateActiveItemError as exc:
        raise HTTPException(
            status.HTTP_409_CONFLICT,
            detail={
                "existing_item": {
                    "id": str(exc.existing_item.id),
                    "name": exc.existing_item.name,
                    "quantity": exc.existing_item.quantity,
                },
                "message": (
                    f"{exc.existing_item.name} is already on the list. "
                    "Add another or update the existing quantity?"
                ),
            },
        ) from exc

    return VoiceRequestAcceptResult(
        voice_request=VoiceRequestResponse.model_validate(request),
        grocery_item=item,
    )


@router.post("/{request_id}/reject", response_model=VoiceRequestResponse)
async def reject_voice_request(
    request_id: UUID,
    db: AsyncSession = Depends(get_db),
    user: AuthUser = Depends(get_current_user),
) -> VoiceRequestResponse:
    service = VoiceService(db)
    try:
        request = await service.reject(request_id)
    except VoiceRequestNotFound as exc:
        raise HTTPException(status.HTTP_404_NOT_FOUND, str(exc)) from exc
    except VoiceRequestAlreadyDecided as exc:
        raise HTTPException(status.HTTP_409_CONFLICT, str(exc)) from exc
    return VoiceRequestResponse.model_validate(request)
