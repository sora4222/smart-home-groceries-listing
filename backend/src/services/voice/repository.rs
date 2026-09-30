//! Every SQL statement the confirmation queue needs.
//!
//! Kept separate from the rules in the parent module so the service reads as
//! policy and this file reads as storage. Functions here take an executor or
//! transaction and make no decisions beyond mapping a missing row to `Option`.
//!
//! Every statement is a literal `&'static str` with bind parameters. sqlx
//! requires that (a runtime-built query string has to be wrapped in
//! `AssertSqlSafe`), and it is worth keeping even where it means repeating a
//! column list: there is then no code path where a query string is assembled
//! at all, so SQL injection is not something a reviewer has to check for.

use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{
    GroceryItem, GroceryItemSource, GroceryItemStatus, IntakeSource, VoiceRequest,
};

/// Collapses whitespace runs and lowercases a name for duplicate comparison.
///
/// This computes the value bound to [`lock_active_duplicate`]; PostgreSQL
/// computes the same form for stored rows, both in that query and in the
/// `ix_grocery_items_normalised_name` index. Keep the three in step — a
/// mismatch silently stops duplicate detection working.
pub fn normalise(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Requests still awaiting a decision, newest first.
pub async fn list_pending<'e, E>(executor: E) -> Result<Vec<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "SELECT id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                grocery_item_id, created_at
         FROM voice_requests
         WHERE status = 'pending'
         ORDER BY created_at DESC",
    )
    .fetch_all(executor)
    .await?)
}

/// Counts requests still awaiting a decision.
pub async fn pending_count<'e, E>(executor: E) -> Result<i64, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(
        sqlx::query_scalar("SELECT count(*) FROM voice_requests WHERE status = 'pending'")
            .fetch_one(executor)
            .await?,
    )
}

/// Whether a request exists at all, used to tell 404 from 409.
pub async fn request_exists<'e, E>(executor: E, request_id: Uuid) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM voice_requests WHERE id = $1")
        .bind(request_id)
        .fetch_one(executor)
        .await?;
    Ok(count > 0)
}

/// Looks up a request by the intake channel's own identifier.
pub async fn find_by_external_id<'e, E>(
    executor: E,
    source: IntakeSource,
    external_id: &str,
) -> Result<Option<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "SELECT id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                grocery_item_id, created_at
         FROM voice_requests
         WHERE source = $1 AND external_id = $2",
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(executor)
    .await?)
}

/// Inserts a new pending request.
pub async fn insert_request<'e, E>(
    executor: E,
    source: IntakeSource,
    external_id: Option<&str>,
    raw_text: &str,
    parsed_name: &str,
    quantity: i32,
) -> Result<VoiceRequest, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "INSERT INTO voice_requests
             (id, source, external_id, raw_text, parsed_name, parsed_quantity, status)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending')
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(source)
    .bind(external_id)
    .bind(raw_text)
    .bind(parsed_name)
    .bind(quantity)
    .fetch_one(executor)
    .await?)
}

/// Marks a pending request rejected. `None` means no pending row matched.
pub async fn reject_if_pending<'e, E>(
    executor: E,
    request_id: Uuid,
) -> Result<Option<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests SET status = 'rejected'
         WHERE id = $1 AND status = 'pending'
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at",
    )
    .bind(request_id)
    .fetch_optional(executor)
    .await?)
}

/// Locks one request row for the rest of the transaction.
pub async fn lock_request(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
) -> Result<Option<VoiceRequest>, ApiError> {
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "SELECT id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                grocery_item_id, created_at
         FROM voice_requests
         WHERE id = $1
         FOR UPDATE",
    )
    .bind(request_id)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Finds an active item whose normalised name matches, locking it so a
/// concurrent merge cannot lose an increment.
///
/// The normalising expression is repeated verbatim in
/// `ix_grocery_items_normalised_name`; that is what lets this use the index
/// instead of scanning every active row.
pub async fn lock_active_duplicate(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<Option<GroceryItem>, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        r"SELECT id, name, quantity, status, source, added_by_user_id, created_at
          FROM grocery_items
          WHERE status = 'active'
            AND lower(btrim(regexp_replace(name, '\s+', ' ', 'g'))) = $1
          ORDER BY created_at
          LIMIT 1
          FOR UPDATE",
    )
    .bind(normalise(name))
    .fetch_optional(&mut **tx)
    .await?)
}

/// Adds a voice-sourced item to the active list.
pub async fn insert_grocery_item(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    quantity: i32,
    user_id: &str,
) -> Result<GroceryItem, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "INSERT INTO grocery_items (id, name, quantity, status, source, added_by_user_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, name, quantity, status, source, added_by_user_id, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(quantity)
    .bind(GroceryItemStatus::Active)
    .bind(GroceryItemSource::Voice)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await?)
}

/// Replaces an item's quantity.
pub async fn set_item_quantity(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
    quantity: i32,
) -> Result<GroceryItem, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "UPDATE grocery_items SET quantity = $2
         WHERE id = $1
         RETURNING id, name, quantity, status, source, added_by_user_id, created_at",
    )
    .bind(item_id)
    .bind(quantity)
    .fetch_one(&mut **tx)
    .await?)
}

/// Records the accepted decision and the item it produced.
pub async fn mark_accepted(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
    name: &str,
    quantity: i32,
    grocery_item_id: Uuid,
) -> Result<VoiceRequest, ApiError> {
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests
         SET status = 'accepted', parsed_name = $2, parsed_quantity = $3, grocery_item_id = $4
         WHERE id = $1
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at",
    )
    .bind(request_id)
    .bind(name)
    .bind(quantity)
    .bind(grocery_item_id)
    .fetch_one(&mut **tx)
    .await?)
}

/// Active list items, newest first.
pub async fn list_active_items<'e, E>(executor: E) -> Result<Vec<GroceryItem>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GroceryItem>(
        "SELECT id, name, quantity, status, source, added_by_user_id, created_at
         FROM grocery_items
         WHERE status = 'active'
         ORDER BY created_at DESC",
    )
    .fetch_all(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::normalise;

    #[test]
    fn normalise_lowercases_and_trims() {
        assert_eq!(normalise("  Toilet Paper  "), "toilet paper");
    }

    #[test]
    fn normalise_collapses_internal_whitespace() {
        assert_eq!(normalise("full   cream   milk"), "full cream milk");
    }

    #[test]
    fn normalise_is_case_and_space_insensitive_for_matching() {
        assert_eq!(normalise("Milk"), normalise("  milk "));
    }

    #[test]
    fn normalise_collapses_tabs_and_newlines() {
        assert_eq!(normalise("full\tcream\nmilk"), "full cream milk");
    }
}
