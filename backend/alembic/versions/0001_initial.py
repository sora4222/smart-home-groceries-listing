"""initial schema: voice_requests, grocery_items

Revision ID: 0001
Revises:
Create Date: 2026-09-26

"""
from typing import Sequence, Union

import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

from alembic import op

revision: str = "0001"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "grocery_items",
        sa.Column(
            "id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False
        ),
        sa.Column("name", sa.String(length=200), nullable=False),
        sa.Column("quantity", sa.Integer(), nullable=False),
        sa.Column(
            "status",
            sa.String(length=20),
            nullable=False,
            server_default="active",
        ),
        sa.Column("source", sa.String(length=20), nullable=False),
        sa.Column("added_by_user_id", sa.String(length=200), nullable=True),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
    )

    op.create_table(
        "voice_requests",
        sa.Column(
            "id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False
        ),
        sa.Column("raw_text", sa.Text(), nullable=False),
        sa.Column("parsed_name", sa.String(length=200), nullable=False),
        sa.Column("parsed_quantity", sa.Integer(), nullable=False),
        sa.Column(
            "status",
            sa.String(length=20),
            nullable=False,
            server_default="pending",
        ),
        sa.Column(
            "grocery_item_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("grocery_items.id"),
            nullable=True,
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
    )
    op.create_index(
        "ix_voice_requests_status", "voice_requests", ["status"]
    )
    op.create_index(
        "ix_grocery_items_status", "grocery_items", ["status"]
    )


def downgrade() -> None:
    op.drop_index("ix_grocery_items_status", table_name="grocery_items")
    op.drop_index("ix_voice_requests_status", table_name="voice_requests")
    op.drop_table("voice_requests")
    op.drop_table("grocery_items")
