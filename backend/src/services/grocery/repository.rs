//! Every SQL statement the grocery list needs.
//!
//! All `grocery_items` storage lives here, including the statements the intake
//! confirmation queue uses when it accepts a request — the table belongs to
//! the list, not to the channel that fed it. `services/voice/repository.rs`
//! therefore calls into this module rather than writing its own item SQL, so
//! the normalising expression duplicate detection depends on has one home.
//!
//! Every statement is a literal `&'static str` with bind parameters; nothing
//! here assembles a query string.

use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemSource, GroceryItemStatus};

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

/// Everything on the household list: items being reviewed and items already
/// committed for purchase, newest first. `ordered` items are history and are
/// left to the purchase-history feature.
pub async fn list_items<'e, E>(executor: E) -> Result<Vec<GroceryItem>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GroceryItem>(
        "SELECT id, name, quantity, status, source, note, filter_terms,
                added_by_user_id, created_at
         FROM grocery_items
         WHERE status IN ('active', 'committed')
         ORDER BY created_at DESC",
    )
    .fetch_all(executor)
    .await?)
}

/// One item by id, whatever its status.
pub async fn find_item<'e, E>(executor: E, item_id: Uuid) -> Result<Option<GroceryItem>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GroceryItem>(
        "SELECT id, name, quantity, status, source, note, filter_terms,
                added_by_user_id, created_at
         FROM grocery_items
         WHERE id = $1",
    )
    .bind(item_id)
    .fetch_optional(executor)
    .await?)
}

/// Locks one item row for the rest of the transaction.
pub async fn lock_item(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
) -> Result<Option<GroceryItem>, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "SELECT id, name, quantity, status, source, note, filter_terms,
                added_by_user_id, created_at
         FROM grocery_items
         WHERE id = $1
         FOR UPDATE",
    )
    .bind(item_id)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Finds an active item whose normalised name matches, locking it so a
/// concurrent merge cannot lose an increment.
///
/// Scoped to `active` on purpose: a committed item is on its way into an order
/// and must not absorb a new request. The normalising expression is repeated
/// verbatim in `ix_grocery_items_normalised_name`; that is what lets this use
/// the index instead of scanning every active row.
pub async fn lock_active_duplicate(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<Option<GroceryItem>, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        r"SELECT id, name, quantity, status, source, note, filter_terms,
                 added_by_user_id, created_at
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

/// Adds an item to the active list.
pub async fn insert_item(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    quantity: i32,
    source: GroceryItemSource,
    note: Option<&str>,
    filter_terms: &[String],
    user_id: &str,
) -> Result<GroceryItem, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "INSERT INTO grocery_items
             (id, name, quantity, status, source, note, filter_terms, added_by_user_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, name, quantity, status, source, note, filter_terms,
                   added_by_user_id, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(quantity)
    .bind(GroceryItemStatus::Active)
    .bind(source)
    .bind(note)
    .bind(filter_terms)
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
         RETURNING id, name, quantity, status, source, note, filter_terms,
                   added_by_user_id, created_at",
    )
    .bind(item_id)
    .bind(quantity)
    .fetch_one(&mut **tx)
    .await?)
}

/// Writes every editable field at once.
///
/// The caller resolves "leave this as it was" before calling, so each bind is
/// the value the row should end up with. One statement rather than one per
/// field keeps a partial edit impossible.
pub async fn update_item(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
    name: &str,
    quantity: i32,
    note: Option<&str>,
    filter_terms: &[String],
) -> Result<GroceryItem, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "UPDATE grocery_items
         SET name = $2, quantity = $3, note = $4, filter_terms = $5
         WHERE id = $1
         RETURNING id, name, quantity, status, source, note, filter_terms,
                   added_by_user_id, created_at",
    )
    .bind(item_id)
    .bind(name)
    .bind(quantity)
    .bind(note)
    .bind(filter_terms)
    .fetch_one(&mut **tx)
    .await?)
}

/// Removes an item. `false` means no row matched.
pub async fn delete_item(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM grocery_items WHERE id = $1")
        .bind(item_id)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Moves every item from one status to another, returning the rows it changed.
///
/// Used for both directions of the commit decision: `active` → `committed`
/// locks the list in for purchase, `committed` → `active` releases it for
/// further editing. One statement means two tabs pressing the button together
/// cannot half-commit the list.
pub async fn move_all<'e, E>(
    executor: E,
    from: GroceryItemStatus,
    to: GroceryItemStatus,
) -> Result<Vec<GroceryItem>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GroceryItem>(
        "UPDATE grocery_items SET status = $2
         WHERE status = $1
         RETURNING id, name, quantity, status, source, note, filter_terms,
                   added_by_user_id, created_at",
    )
    .bind(from)
    .bind(to)
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
