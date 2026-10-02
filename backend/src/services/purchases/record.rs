//! Saving a filled trolley as bought.
//!
//! When the "Fill trolley" bookmarklet reports, every product it added is
//! saved as bought: priced ([`super::price`]), given its share of the
//! delivery fee ([`super::fee`]) and a category ([`super::category`]). The
//! list items it fulfilled become `ordered`, so they leave the list. A product
//! that could not be added stays on the list and is not saved.
//!
//! The stores are asked before the transaction opens, so no row is locked
//! while a store answers.

use std::collections::HashMap;

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use super::price::{price_line, BoughtLine};
use super::repository::{self, NewOrder, NewPurchase};
use super::{category, fee, log};
use crate::error::ApiError;
use crate::models::db::{
    GroceryItemStatus, PurchaseOrder, TrolleyHandoff, TrolleyHandoffLine, TrolleyLineOutcome,
};
use crate::services::grocery::repository as items;
use crate::services::product_search::ProductSearchService;
use crate::services::selections::repository as selections;
use crate::services::stores::StoreClients;
use crate::services::trolley_handoffs::{line_repository, repository as handoffs};

/// Saves the products a reported handoff added as one order.
///
/// `None` when nothing was saved: no product was added, none could be priced,
/// or this handoff was saved already.
pub async fn trolley_fill(
    pool: &PgPool,
    stores: &StoreClients,
    handoff_id: Uuid,
) -> Result<Option<PurchaseOrder>, ApiError> {
    let Some(handoff) = handoffs::find_handoff(pool, handoff_id).await? else {
        return Ok(None);
    };
    let added: Vec<TrolleyHandoffLine> = line_repository::lines_of(pool, handoff_id)
        .await?
        .into_iter()
        .filter(|line| line.outcome == Some(TrolleyLineOutcome::Added))
        .collect();
    let bought = price_all(pool, stores, &handoff, &added).await?;
    if bought.is_empty() {
        log::nothing_to_save(&handoff, added.len());
        return Ok(None);
    }
    save(pool, &handoff, &bought).await
}

/// Prices every added line whose list item is still to be bought.
async fn price_all(
    pool: &PgPool,
    stores: &StoreClients,
    handoff: &TrolleyHandoff,
    added: &[TrolleyHandoffLine],
) -> Result<Vec<BoughtLine>, ApiError> {
    let mut choices: HashMap<Uuid, _> = selections::list_all(pool)
        .await?
        .into_iter()
        .map(|choice| (choice.grocery_item_id, choice))
        .collect();
    let search = ProductSearchService::new(pool, stores);
    let mut bought = Vec::new();
    for line in added {
        let item = items::find_item(pool, line.grocery_item_id).await?;
        let Some(item) = item.filter(|item| still_to_buy(item.status)) else {
            log::line_skipped(line, "no longer on the list");
            continue;
        };
        let outcome = search.search_at(&item, handoff.store).await;
        let choice = choices.remove(&item.id);
        match price_line(
            handoff.store,
            line,
            &item,
            choice.as_ref(),
            outcome.as_ref(),
        ) {
            Some(priced) => bought.push(priced),
            None => log::line_skipped(line, "no price from the store or the choice"),
        }
    }
    Ok(bought)
}

/// Writes the order, its purchases and the items' new status together.
async fn save(
    pool: &PgPool,
    handoff: &TrolleyHandoff,
    bought: &[BoughtLine],
) -> Result<Option<PurchaseOrder>, ApiError> {
    let totals: Vec<Decimal> = bought.iter().map(|line| line.total_price).collect();
    let delivery_fee = handoff.delivery_fee.unwrap_or(Decimal::ZERO);
    let shares = fee::split(delivery_fee, &totals);

    let mut tx = pool.begin().await?;
    let new_order = NewOrder {
        store: handoff.store,
        items_total: totals.iter().sum(),
        delivery_fee,
        recorded_by: &handoff.created_by,
        trolley_handoff_id: handoff.id,
        bought_at: handoff.reported_at.unwrap_or_else(Utc::now),
    };
    let Some(order) = repository::insert_order(&mut *tx, &new_order).await? else {
        tx.rollback().await?;
        log::already_saved(handoff.id);
        return Ok(None);
    };
    for (line, share) in bought.iter().zip(shares) {
        let item_key = items::normalise(&line.item_name);
        let found = category::categorise(line.store_category.as_deref(), &line.product_name);
        let purchase = NewPurchase {
            grocery_item_id: line.grocery_item_id,
            item_name: &line.item_name,
            item_key: &item_key,
            item_status_before: line.item_status_before,
            store: line.store,
            product_id: &line.product_id,
            product_name: &line.product_name,
            brand: line.brand.as_deref(),
            package_size: line.package_size.as_deref(),
            quantity: line.quantity,
            unit_price: line.unit_price,
            shelf_price: line.shelf_price,
            total_price: line.total_price,
            delivery_fee_share: share,
            store_category: line.store_category.as_deref(),
            category: found.category,
            category_source: found.source,
        };
        repository::insert_purchase(&mut *tx, &order, &purchase).await?;
    }
    let ids: Vec<Uuid> = bought.iter().map(|line| line.grocery_item_id).collect();
    let moved = items::mark_ordered(&mut *tx, &ids).await?;
    tx.commit().await?;
    log::saved(&order, bought.len(), moved.len());
    Ok(Some(order))
}

/// Only items still waiting to be bought are saved as bought.
fn still_to_buy(status: GroceryItemStatus) -> bool {
    matches!(
        status,
        GroceryItemStatus::Active | GroceryItemStatus::Committed
    )
}
