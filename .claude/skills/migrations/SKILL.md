---
name: migrations
description: Change the PostgreSQL schema through this Rust/sqlx project's embedded SQL migrations. Use for schema, index, or persisted-data changes; not for ordinary query edits.
---

# sqlx migrations

Migrations are forward-only SQL files in `backend/migrations/`, embedded by
`sqlx::migrate!`. Create one with `make migration-new name="description"` and
apply it with `make migrate`.

- Never edit a migration that may have been applied: sqlx verifies checksums.
  Add a new migration instead.
- Use literal SQL and bind parameters in application queries; schema
  constraints complement service validation.
- Keep a schema change and its affected row types, repository queries, and
  integration tests consistent. Use a transaction by default; use the
  project's `.no-tx.sql` convention only when PostgreSQL requires it.
- Prefer `TEXT` with `CHECK` constraints to PostgreSQL enums. Add indexes for
  real query predicates, and verify any expression index matches its query.
- Validate with the relevant `#[sqlx::test]` coverage against a disposable
  database. Back up before a destructive production migration.
