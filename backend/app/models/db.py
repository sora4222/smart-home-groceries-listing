"""SQLAlchemy ORM models.

Only the tables needed for the Google Home voice-intake goal are defined
here (`voice_requests`, `grocery_items`). Later features (item_selections,
item_rules, purchase history, etc.) add their own tables alongside these
without changing this file's existing columns.
"""

import uuid
from enum import StrEnum

from sqlalchemy import Enum, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


def _uuid_pk() -> Mapped[uuid.UUID]:
    return mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid.uuid4
    )


class VoiceRequestStatus(StrEnum):
    PENDING = "pending"
    ACCEPTED = "accepted"
    REJECTED = "rejected"


class GroceryItemStatus(StrEnum):
    PENDING = "pending"
    ACTIVE = "active"
    ORDERED = "ordered"


class GroceryItemSource(StrEnum):
    VOICE = "voice"
    MANUAL = "manual"


class VoiceRequest(TimestampMixin, Base):
    """A raw voice-added item, unconfirmed until a household member acts on it."""

    __tablename__ = "voice_requests"

    id: Mapped[uuid.UUID] = _uuid_pk()
    raw_text: Mapped[str] = mapped_column(Text, nullable=False)
    parsed_name: Mapped[str] = mapped_column(String(200), nullable=False)
    parsed_quantity: Mapped[int] = mapped_column(Integer, nullable=False, default=1)
    status: Mapped[VoiceRequestStatus] = mapped_column(
        Enum(VoiceRequestStatus, native_enum=False, length=20),
        nullable=False,
        default=VoiceRequestStatus.PENDING,
    )
    grocery_item_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("grocery_items.id"), nullable=True
    )

    grocery_item: Mapped["GroceryItem | None"] = relationship(
        back_populates="voice_request"
    )


class GroceryItem(TimestampMixin, Base):
    """An item on the shared household grocery list."""

    __tablename__ = "grocery_items"

    id: Mapped[uuid.UUID] = _uuid_pk()
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    quantity: Mapped[int] = mapped_column(Integer, nullable=False, default=1)
    status: Mapped[GroceryItemStatus] = mapped_column(
        Enum(GroceryItemStatus, native_enum=False, length=20),
        nullable=False,
        default=GroceryItemStatus.ACTIVE,
    )
    source: Mapped[GroceryItemSource] = mapped_column(
        Enum(GroceryItemSource, native_enum=False, length=20), nullable=False
    )
    added_by_user_id: Mapped[str | None] = mapped_column(String(200), nullable=True)

    voice_request: Mapped["VoiceRequest | None"] = relationship(
        back_populates="grocery_item"
    )
