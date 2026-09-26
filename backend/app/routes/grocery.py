"""Active grocery list — read access needed by the Pending Requests flow
(duplicate detection surfaces the existing active item) plus the grocery
list view itself.
"""

from fastapi import APIRouter, Depends
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.provider import AuthUser, get_current_user
from app.db.session import get_db
from app.models.db import GroceryItem, GroceryItemStatus
from app.models.schemas import GroceryItemResponse

router = APIRouter(prefix="/api/grocery-items", tags=["grocery"])


@router.get("", response_model=list[GroceryItemResponse])
async def list_active_items(
    db: AsyncSession = Depends(get_db),
    user: AuthUser = Depends(get_current_user),
) -> list[GroceryItemResponse]:
    result = await db.execute(
        select(GroceryItem)
        .where(GroceryItem.status == GroceryItemStatus.ACTIVE)
        .order_by(GroceryItem.created_at.desc())
    )
    return [GroceryItemResponse.model_validate(row) for row in result.scalars()]
