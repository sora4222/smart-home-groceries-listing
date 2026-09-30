# Migrations (sqlx)

Migrations are plain SQL files in `backend/migrations/`, embedded into the
binary at compile time by `sqlx::migrate!` and applied at startup. There is no
Alembic, no autogenerate, and no ORM metadata to keep in step — the SQL *is*
the schema.

## Workflow
```bash
# 1. Create the file
make migration-new name="add item rules table"
#    → backend/migrations/0002_add_item_rules_table.sql

# 2. Write the SQL by hand (see the checklist below)

# 3. Add or change the matching row type in backend/src/models/db.rs
#    and the SQL in the domain's repository.rs

# 4. Apply it
make migrate

# 5. Run the tests — #[sqlx::test] applies migrations to a fresh database
#    per test, so a broken migration fails the whole suite immediately
make test-backend
```

## Rules
- One migration per logical schema change.
- **Never edit an applied migration.** `sqlx` stores a checksum per version in
  `_sqlx_migrations`; changing a file that has run makes the backend refuse to
  start with a checksum mismatch. Write a new migration instead.
- Numbering is sequential and gapless: `0001_`, `0002_`, .... Check the highest
  existing file first (`make migration-new` does).
- Migrations run inside a transaction by default. For something that cannot
  (`CREATE INDEX CONCURRENTLY`, `ALTER TYPE ... ADD VALUE`), name the file
  `<version>_<description>.no-tx.sql`.
- Prefer `TEXT` + `CHECK` over PostgreSQL `ENUM`. Adding a value is then a
  constraint change rather than an `ALTER TYPE` that cannot run in a
  transaction.
- Put the constraints in the schema, not only in `validator`: a length or
  range limit belongs in both, so a bug in a service cannot write a bad row.
- Reversibility: `sqlx` supports `.up.sql`/`.down.sql` pairs, but this project
  uses forward-only plain `.sql`. Recovering from a bad migration means writing
  the next one. For anything destructive, dump first.

## File checklist
```sql
-- 0002_add_item_rules.sql
--
-- Say what this is for in a comment. A migration is read far more often
-- than it is written, usually by someone debugging.

CREATE TABLE item_rules (
    id              UUID        PRIMARY KEY,
    trigger         TEXT        NOT NULL CHECK (char_length(trigger) BETWEEN 1 AND 200),
    filter_terms    TEXT[]      NOT NULL DEFAULT '{}',
    apply_to_manual BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_item_rules_trigger ON item_rules (trigger);
```

- Every table: `id UUID PRIMARY KEY`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
- Every status/source column: `TEXT` with an explicit `CHECK (... IN (...))`.
- Index anything a `WHERE` clause filters on. Partial (`WHERE status = 'active'`)
  where only some rows are ever queried.

## Expression indexes must match their query
`ix_grocery_items_normalised_name` indexes
`lower(btrim(regexp_replace(name, '\s+', ' ', 'g')))`. The identical
expression appears in `repository::lock_active_duplicate`, and
`repository::normalise` in Rust computes the value bound to it. Change one and
you must change all three — otherwise duplicate detection silently stops
working, or quietly starts a sequential scan. There is a unit test on
`normalise`; there is nothing that can catch the index drifting, so check it
by hand with `EXPLAIN`.

## Useful commands
```bash
# Current state
psql "$DATABASE_URL" -c "select version, description, success from _sqlx_migrations order by version;"

# Apply without starting the server
make migrate

# Verify an expression index is actually used
psql "$DATABASE_URL" -c "explain analyze select * from grocery_items
  where status = 'active'
    and lower(btrim(regexp_replace(name, '\s+', ' ', 'g'))) = 'full cream milk';"
```

## Model → table mapping
Row types live in `backend/src/models/db.rs`; request/response bodies in
`backend/src/models/schemas.rs`; the SQL in each domain's
`services/<domain>/repository.rs`. No business logic in any of the three.
