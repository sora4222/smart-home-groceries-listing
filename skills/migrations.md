# Alembic Migrations

## Workflow
```bash
# 1. Edit the SQLAlchemy model in backend/app/models/db.py
# 2. Generate migration
make migration-new name="add item rules table"
# 3. Review the generated file in backend/alembic/versions/
# 4. Apply
make migrate
# 5. Run tests
make test-backend
```

## Rules
- One migration per logical schema change
- Never edit an applied migration — create a new one
- Always review the auto-generated diff before applying
- Always implement `downgrade()` — never leave it as `pass`
- Column renames: auto-generate creates drop+add — change to `op.alter_column` manually

## Generated file checklist
```python
def upgrade() -> None:
    op.create_table('item_rules',
        sa.Column('id', sa.UUID(), nullable=False),
        sa.Column('trigger', sa.String(), nullable=False),
        sa.Column('filter_terms', sa.ARRAY(sa.String()), nullable=False),
        sa.Column('apply_to_manual', sa.Boolean(), server_default='false'),
        sa.PrimaryKeyConstraint('id'),
    )

def downgrade() -> None:
    op.drop_table('item_rules')   # ← must always be present
```

## Useful commands
```bash
# Check current state
docker compose exec backend uv run alembic current

# Show history
docker compose exec backend uv run alembic history --verbose

# Rollback one migration
docker compose exec backend uv run alembic downgrade -1

# Rollback to base (empty DB)
docker compose exec backend uv run alembic downgrade base
```

## Model → table mapping
SQLAlchemy models live in `backend/app/models/db.py`.
Pydantic schemas (request/response) live in `backend/app/models/schemas.py`.
Do not put business logic in either file.
