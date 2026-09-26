"""Business logic for the voice-request confirmation queue.

Routes stay thin (HTTP concerns only); this service owns the rules from
the spec's "Google Home Integration" and "Duplicate handling" sections.
"""

from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.db import (
    GroceryItem,
    GroceryItemSource,
    GroceryItemStatus,
    VoiceRequest,
    VoiceRequestStatus,
)
from app.models.schemas import VoiceRequestCreate, VoiceRequestDecision
from app.services.ws_manager import manager


class DuplicateActiveItemError(Exception):
    """Raised when accepting a request would duplicate an active item."""

    def __init__(self, existing_item: GroceryItem) -> None:
        self.existing_item = existing_item
        super().__init__(f"{existing_item.name!r} is already on the list")


class VoiceRequestNotFound(Exception):
    pass


class VoiceRequestAlreadyDecided(Exception):
    pass


def _normalise(name: str) -> str:
    return " ".join(name.strip().lower().split())


class VoiceService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def create_request(self, payload: VoiceRequestCreate) -> VoiceRequest:
        """Record a new unconfirmed voice request and notify open browser sessions."""
        request = VoiceRequest(
            raw_text=payload.item,
            parsed_name=payload.item.strip(),
            parsed_quantity=payload.quantity,
            status=VoiceRequestStatus.PENDING,
        )
        self.db.add(request)
        await self.db.commit()
        await self.db.refresh(request)

        pending_count = await self._pending_count()
        await manager.broadcast(
            {"type": "voice_request_added", "count": pending_count}
        )
        return request

    async def list_pending(self) -> list[VoiceRequest]:
        result = await self.db.execute(
            select(VoiceRequest)
            .where(VoiceRequest.status == VoiceRequestStatus.PENDING)
            .order_by(VoiceRequest.created_at.desc())
        )
        return list(result.scalars())

    async def _pending_count(self) -> int:
        result = await self.db.execute(
            select(VoiceRequest).where(VoiceRequest.status == VoiceRequestStatus.PENDING)
        )
        return len(result.scalars().all())

    async def _get_pending(self, request_id: UUID) -> VoiceRequest:
        request = await self.db.get(VoiceRequest, request_id)
        if request is None:
            raise VoiceRequestNotFound(str(request_id))
        if request.status != VoiceRequestStatus.PENDING:
            raise VoiceRequestAlreadyDecided(str(request_id))
        return request

    async def _get_acceptable(self, request_id: UUID) -> VoiceRequest:
        """Pending or rejected requests can be accepted.

        Rejected is included so the web app's "dulled, undo-on-hover" reject
        UX (see FEATURE_VOICE.md) can turn a rejection back into an accept.
        Only an already-accepted request is a truly closed decision.
        """
        request = await self.db.get(VoiceRequest, request_id)
        if request is None:
            raise VoiceRequestNotFound(str(request_id))
        if request.status == VoiceRequestStatus.ACCEPTED:
            raise VoiceRequestAlreadyDecided(str(request_id))
        return request

    async def _find_active_duplicate(self, name: str) -> GroceryItem | None:
        target = _normalise(name)
        result = await self.db.execute(
            select(GroceryItem).where(GroceryItem.status == GroceryItemStatus.ACTIVE)
        )
        for item in result.scalars():
            if _normalise(item.name) == target:
                return item
        return None

    async def accept(
        self,
        request_id: UUID,
        decision: VoiceRequestDecision,
        user_id: str,
        merge_with_duplicate: bool = False,
    ) -> tuple[VoiceRequest, GroceryItem]:
        """Accept a pending request, applying any corrections, into the active list.

        Raises `DuplicateActiveItemError` if an active item of the same
        (normalised) name already exists and `merge_with_duplicate` is False,
        so the caller can offer the user a choice per the spec's duplicate
        handling rule. Passing `merge_with_duplicate=True` increments the
        existing item's quantity instead of creating a new row.
        """
        request = await self._get_acceptable(request_id)
        name = decision.name or request.parsed_name
        quantity = decision.quantity or request.parsed_quantity

        duplicate = await self._find_active_duplicate(name)
        if duplicate is not None and not merge_with_duplicate:
            raise DuplicateActiveItemError(duplicate)

        if duplicate is not None and merge_with_duplicate:
            duplicate.quantity += quantity
            grocery_item = duplicate
        else:
            grocery_item = GroceryItem(
                name=name,
                quantity=quantity,
                status=GroceryItemStatus.ACTIVE,
                source=GroceryItemSource.VOICE,
                added_by_user_id=user_id,
            )
            self.db.add(grocery_item)

        request.status = VoiceRequestStatus.ACCEPTED
        request.parsed_name = name
        request.parsed_quantity = quantity

        await self.db.flush()
        request.grocery_item_id = grocery_item.id
        await self.db.commit()
        await self.db.refresh(request)
        await self.db.refresh(grocery_item)

        await manager.broadcast(
            {"type": "voice_request_added", "count": await self._pending_count()}
        )
        return request, grocery_item

    async def reject(self, request_id: UUID) -> VoiceRequest:
        """Reject a pending request. Rejections are logged, not deleted."""
        request = await self._get_pending(request_id)
        request.status = VoiceRequestStatus.REJECTED
        await self.db.commit()
        await self.db.refresh(request)

        await manager.broadcast(
            {"type": "voice_request_added", "count": await self._pending_count()}
        )
        return request
