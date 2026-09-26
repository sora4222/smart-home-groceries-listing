"""Pydantic request/response schemas for the voice-intake and grocery-list API."""

from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from app.models.db import GroceryItemSource, GroceryItemStatus, VoiceRequestStatus


class VoiceRequestCreate(BaseModel):
    """Payload the Google Home Action posts to `/api/voice-requests`."""

    item: str = Field(min_length=1, max_length=200)
    quantity: int = Field(default=1, ge=1, le=999)


class VoiceRequestResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: UUID
    raw_text: str
    parsed_name: str
    parsed_quantity: int
    status: VoiceRequestStatus
    created_at: datetime


class VoiceRequestDecision(BaseModel):
    """Optional corrections a user makes before accepting a voice request."""

    name: str | None = Field(default=None, min_length=1, max_length=200)
    quantity: int | None = Field(default=None, ge=1, le=999)


class VoiceRequestAcceptResult(BaseModel):
    """Response after a voice request is accepted: the resulting grocery item."""

    voice_request: VoiceRequestResponse
    grocery_item: "GroceryItemResponse"


class GroceryItemResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: UUID
    name: str
    quantity: int
    status: GroceryItemStatus
    source: GroceryItemSource
    added_by_user_id: str | None
    created_at: datetime


class NewGroceryItem(BaseModel):
    """Payload for a manual (web app) grocery item addition."""

    name: str = Field(min_length=1, max_length=200)
    quantity: int = Field(default=1, ge=1, le=999)


class DuplicateItemWarning(BaseModel):
    """Returned instead of a 201 when an active item with the same name exists."""

    existing_item: GroceryItemResponse
    message: str


VoiceRequestAcceptResult.model_rebuild()
