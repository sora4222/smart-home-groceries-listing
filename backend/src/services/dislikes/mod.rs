//! Product dislikes: per member, seen by the whole household.
//!
//! A member marks a store product as disliked. Everyone sees who disliked it
//! when they compare prices, but the product stays choosable — a person may
//! buy it anyway. A dislike can be set aside two ways:
//! - **for this order** — an override on one list item ([`DislikeService::override_for_item`]);
//!   the dislike stays, and the next time the item is added it counts again;
//! - **for good** — the member removes their own dislike ([`DislikeService::remove_mine`]).
//!   Nobody can remove another member's dislike.
//!
//! The automated order (the optimiser, not built yet) reads
//! [`DislikeService::book_for_item`] and decides with [`skip`].
//!
//! SQL lives in [`repository`] and [`overrides_repository`].

mod log;
mod member_name;
pub mod overrides_repository;
pub mod repository;
pub mod skip;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{DislikeOverride, GroceryItemStatus, ProductDislike};
use crate::services::grocery::repository as items;
use crate::services::stores::Store;
pub use member_name::member_name;
use repository::NewDislike;
use skip::DislikeBook;

/// The member a dislike or an override is for.
#[derive(Debug, Clone, Copy)]
pub struct Member<'a> {
    pub user_id: &'a str,
    pub email: Option<&'a str>,
}

/// What a member says they dislike, as the price comparison showed it.
#[derive(Debug, Clone, Copy)]
pub struct DislikedProduct<'a> {
    pub store: Store,
    pub product_id: &'a str,
    pub name: &'a str,
    pub brand: Option<&'a str>,
    pub package_size: Option<&'a str>,
}

/// Reads and writes dislikes and their overrides.
pub struct DislikeService<'a> {
    pool: &'a PgPool,
}

impl<'a> DislikeService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Every member's dislikes, newest first.
    pub async fn list_household(&self) -> Result<Vec<ProductDislike>, ApiError> {
        repository::list_all(self.pool).await
    }

    /// Saves `member`'s dislike of the product. Disliking again is not an
    /// error; it refreshes the labels. 422 when the name is only spaces.
    pub async fn dislike(
        &self,
        member: Member<'_>,
        product: DislikedProduct<'_>,
    ) -> Result<ProductDislike, ApiError> {
        let name = product.name.trim();
        if name.is_empty() {
            return Err(ApiError::UnprocessableEntity(
                "Say which product is disliked: its name is empty.".into(),
            ));
        }
        let user_name = member_name(member.email);
        let saved = repository::upsert(
            self.pool,
            &NewDislike {
                user_id: member.user_id,
                user_name: &user_name,
                store: product.store,
                product_id: product.product_id,
                product_name: name,
                brand: blank_to_none(product.brand),
                package_size: blank_to_none(product.package_size),
            },
        )
        .await?;
        log::disliked(&saved);
        Ok(saved)
    }

    /// Removes `user_id`'s own dislike: the spec's permanent override. Other
    /// members' dislikes of the same product stay.
    pub async fn remove_mine(
        &self,
        user_id: &str,
        store: Store,
        product_id: &str,
    ) -> Result<(), ApiError> {
        let had = repository::delete_for_user(self.pool, user_id, store, product_id).await?;
        log::removed(user_id, store, product_id, had);
        Ok(())
    }

    /// Every "buy it this time anyway" on the list.
    pub async fn list_overrides(&self) -> Result<Vec<DislikeOverride>, ApiError> {
        overrides_repository::list_all(self.pool).await
    }

    /// Sets the product's dislikes aside for this one list item.
    ///
    /// 404 unknown item; 409 the item has been ordered or removed from the
    /// list; 422 nobody dislikes the product, so there is nothing to override.
    pub async fn override_for_item(
        &self,
        item_id: Uuid,
        store: Store,
        product_id: &str,
        user_id: &str,
    ) -> Result<DislikeOverride, ApiError> {
        let mut tx = self.pool.begin().await?;
        let item = items::lock_item(&mut tx, item_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Grocery item not found".into()))?;
        if !matches!(
            item.status,
            GroceryItemStatus::Active | GroceryItemStatus::Committed
        ) {
            log::override_refused(item_id, store, product_id, "item is not on the list");
            return Err(ApiError::Conflict(format!(
                "{} is no longer on the list, so its order cannot change.",
                item.name
            )));
        }
        if repository::list_for_product(&mut *tx, store, product_id)
            .await?
            .is_empty()
        {
            log::override_refused(item_id, store, product_id, "nobody dislikes it");
            return Err(ApiError::UnprocessableEntity(
                "Nobody dislikes that product, so there is nothing to override.".into(),
            ));
        }
        let saved =
            overrides_repository::upsert(&mut *tx, item_id, store, product_id, user_id).await?;
        tx.commit().await?;
        log::overridden(&saved);
        Ok(saved)
    }

    /// Takes an override back, so the dislikes count again for the item.
    pub async fn clear_override(
        &self,
        item_id: Uuid,
        store: Store,
        product_id: &str,
    ) -> Result<(), ApiError> {
        let had = overrides_repository::delete(self.pool, item_id, store, product_id).await?;
        log::override_cleared(item_id, store, product_id, had);
        Ok(())
    }

    /// Everything the automated order needs to respect dislikes for one item:
    /// every member's dislikes and the item's own overrides. Decide with
    /// [`skip::pick_for_automated_order`].
    pub async fn book_for_item(&self, item_id: Uuid) -> Result<DislikeBook, ApiError> {
        let dislikes = repository::list_all(self.pool).await?;
        let overrides = overrides_repository::list_for_item(self.pool, item_id).await?;
        Ok(DislikeBook::new(dislikes, overrides))
    }
}

/// A label that is only spaces is no label.
fn blank_to_none(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}
