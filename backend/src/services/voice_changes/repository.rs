//! Every SQL statement voice list changes need.
//!
//! Statements on `grocery_items` that other features also use (changing a
//! quantity, deleting a row) stay in `services/grocery/repository.rs`; this
//! module holds the change records, the name lookup, and the re-insert Undo
//! needs. Every statement is a literal with bind parameters.

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, IntakeSource, VoiceChangeKind, VoiceListChange};

/// Takes the one lock every voice change and Undo holds until it commits.
///
/// Changes and Undos are rare and quick, so running them one at a time costs
/// nothing and means a retried request always sees the first one's result.
pub async fn serialise(tx: &mut Transaction<'_, Postgres>) -> Result<(), ApiError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('voice_list_changes'))")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// The change a request id already made, if any.
pub async fn find_by_external_id<'e, E>(
    executor: E,
    source: IntakeSource,
    external_id: &str,
) -> Result<Option<VoiceListChange>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceListChange>(
        "SELECT id, source, external_id, kind, grocery_item_id, item_name, quantity_before,
                quantity_after, item_source, item_note, item_filter_terms, item_added_by_user_id,
                item_created_at, created_at, undone_at, undo_external_id
         FROM voice_list_changes
         WHERE source = $1 AND external_id = $2",
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(executor)
    .await?)
}

/// The change an Undo request id already reversed, if any.
pub async fn find_by_undo_external_id<'e, E>(
    executor: E,
    source: IntakeSource,
    undo_external_id: &str,
) -> Result<Option<VoiceListChange>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceListChange>(
        "SELECT id, source, external_id, kind, grocery_item_id, item_name, quantity_before,
                quantity_after, item_source, item_note, item_filter_terms, item_added_by_user_id,
                item_created_at, created_at, undone_at, undo_external_id
         FROM voice_list_changes
         WHERE source = $1 AND undo_external_id = $2",
    )
    .bind(source)
    .bind(undo_external_id)
    .fetch_optional(executor)
    .await?)
}

/// Locks the list item a spoken name means, or `None`.
///
/// `variants` comes from `matching::name_variants`, exact form first. An exact
/// match wins over a plural one, then an active item over a committed one,
/// then the oldest. Committed items are found too, so the caller can say the
/// list is locked rather than that the item is missing. The normalising
/// expression is the one `ix_grocery_items_normalised_name` indexes.
pub async fn lock_item_named(
    tx: &mut Transaction<'_, Postgres>,
    variants: &[String],
) -> Result<Option<GroceryItem>, ApiError> {
    let Some(exact) = variants.first() else {
        return Ok(None);
    };
    Ok(sqlx::query_as::<_, GroceryItem>(
        r"SELECT id, name, quantity, status, source, note, filter_terms,
                 added_by_user_id, created_at
          FROM grocery_items
          WHERE status IN ('active', 'committed')
            AND lower(btrim(regexp_replace(name, '\s+', ' ', 'g'))) = ANY($1)
          ORDER BY lower(btrim(regexp_replace(name, '\s+', ' ', 'g'))) = $2 DESC,
                   status = 'active' DESC,
                   created_at
          LIMIT 1
          FOR UPDATE",
    )
    .bind(variants)
    .bind(exact)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Records a change made to `item`, snapshotting the item as it was.
///
/// `quantity_after` of `0` records a removal; anything else a reduction.
pub async fn insert_change(
    tx: &mut Transaction<'_, Postgres>,
    source: IntakeSource,
    external_id: Option<&str>,
    item: &GroceryItem,
    quantity_after: i32,
) -> Result<VoiceListChange, ApiError> {
    let kind = if quantity_after == 0 {
        VoiceChangeKind::Removed
    } else {
        VoiceChangeKind::Reduced
    };
    Ok(sqlx::query_as::<_, VoiceListChange>(
        "INSERT INTO voice_list_changes
             (id, source, external_id, kind, grocery_item_id, item_name, quantity_before,
              quantity_after, item_source, item_note, item_filter_terms,
              item_added_by_user_id, item_created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING id, source, external_id, kind, grocery_item_id, item_name,
                   quantity_before, quantity_after, item_source, item_note,
                   item_filter_terms, item_added_by_user_id, item_created_at, created_at,
                   undone_at, undo_external_id",
    )
    .bind(Uuid::new_v4())
    .bind(source)
    .bind(external_id)
    .bind(kind)
    .bind(item.id)
    .bind(&item.name)
    .bind(item.quantity)
    .bind(quantity_after)
    .bind(item.source)
    .bind(&item.note)
    .bind(&item.filter_terms)
    .bind(&item.added_by_user_id)
    .bind(item.created_at)
    .fetch_one(&mut **tx)
    .await?)
}

/// Locks the newest change from `source` made since `since` that still stands.
pub async fn lock_newest_standing(
    tx: &mut Transaction<'_, Postgres>,
    source: IntakeSource,
    since: DateTime<Utc>,
) -> Result<Option<VoiceListChange>, ApiError> {
    Ok(sqlx::query_as::<_, VoiceListChange>(
        "SELECT id, source, external_id, kind, grocery_item_id, item_name, quantity_before,
                quantity_after, item_source, item_note, item_filter_terms, item_added_by_user_id,
                item_created_at, created_at, undone_at, undo_external_id
         FROM voice_list_changes
         WHERE source = $1 AND undone_at IS NULL AND created_at >= $2
         ORDER BY created_at DESC
         LIMIT 1
         FOR UPDATE",
    )
    .bind(source)
    .bind(since)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Marks a change as reversed by the Undo with `undo_external_id`.
pub async fn mark_undone(
    tx: &mut Transaction<'_, Postgres>,
    change_id: Uuid,
    undo_external_id: Option<&str>,
) -> Result<VoiceListChange, ApiError> {
    Ok(sqlx::query_as::<_, VoiceListChange>(
        "UPDATE voice_list_changes SET undone_at = now(), undo_external_id = $2
         WHERE id = $1
         RETURNING id, source, external_id, kind, grocery_item_id, item_name,
                   quantity_before, quantity_after, item_source, item_note,
                   item_filter_terms, item_added_by_user_id, item_created_at, created_at,
                   undone_at, undo_external_id",
    )
    .bind(change_id)
    .bind(undo_external_id)
    .fetch_one(&mut **tx)
    .await?)
}

/// Puts a removed item back on the list as it was: same id, name, quantity,
/// note, chips and place in the list. It comes back `active`.
pub async fn restore_item(
    tx: &mut Transaction<'_, Postgres>,
    change: &VoiceListChange,
) -> Result<GroceryItem, ApiError> {
    Ok(sqlx::query_as::<_, GroceryItem>(
        "INSERT INTO grocery_items
             (id, name, quantity, status, source, note, filter_terms, added_by_user_id,
              created_at)
         VALUES ($1, $2, $3, 'active', $4, $5, $6, $7, $8)
         RETURNING id, name, quantity, status, source, note, filter_terms,
                   added_by_user_id, created_at",
    )
    .bind(change.grocery_item_id)
    .bind(&change.item_name)
    .bind(change.quantity_before)
    .bind(change.item_source)
    .bind(&change.item_note)
    .bind(&change.item_filter_terms)
    .bind(&change.item_added_by_user_id)
    .bind(change.item_created_at)
    .fetch_one(&mut **tx)
    .await?)
}
