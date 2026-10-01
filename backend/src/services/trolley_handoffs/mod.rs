//! Handing chosen products to a store's online trolley.
//!
//! The server cannot log in to a store on its own (Woolworths login uses
//! passkeys and MFA in JavaScript), so filling the trolley is handed to the
//! household's own browser tab, which is already logged in:
//!
//! 1. The web app creates a handoff, with the delivery time wanted
//!    (default: the next day — see [`delivery`]) ([`TrolleyHandoffService::create_for_store`]):
//!    every item still to be bought whose chosen product is at that store.
//! 2. The "Fill trolley" bookmarklet, run on the store's website, claims the
//!    newest waiting handoff ([`TrolleyHandoffService::claim_for_store_tab`]).
//! 3. It reserves a delivery window on the store's website (changeable there
//!    later), adds each product through the website's own trolley call, then
//!    reports what happened ([`TrolleyHandoffService::record_store_tab_report`]).
//! 4. The web app shows the result ([`TrolleyHandoffService::find`]).
//!
//! A handoff can be claimed for [`HANDOFF_LIFETIME_MINUTES`]; creating a new
//! one replaces any still waiting for the same store. Nothing is bought here:
//! the household reviews the trolley and pays on the store's own website.
//! SQL lives in [`repository`]; the store-tab rules in [`store_tab`].

pub mod delivery;
pub mod line_repository;
mod log;
pub mod repository;
pub mod store_tab;

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{TrolleyHandoff, TrolleyHandoffLine, TrolleyHandoffStatus};
use crate::services::stores::Store;
use delivery::{DeliveryRequest, ReportedDelivery};
use store_tab::{ReportedLine, StoreTabLine};

/// How long a new handoff waits for the store tab to claim it.
pub const HANDOFF_LIFETIME_MINUTES: i64 = 30;

/// A handoff with its lines.
#[derive(Debug, Clone)]
pub struct HandoffWithLines {
    pub handoff: TrolleyHandoff,
    pub lines: Vec<TrolleyHandoffLine>,
}

/// A claimed handoff, as the store tab receives it.
#[derive(Debug, Clone)]
pub struct ClaimedHandoff {
    pub handoff: TrolleyHandoff,
    pub lines: Vec<StoreTabLine>,
}

/// Creates, claims and closes trolley handoffs.
pub struct TrolleyHandoffService<'a> {
    pool: &'a PgPool,
}

impl<'a> TrolleyHandoffService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Creates a handoff of every item still to be bought whose chosen
    /// product is at `store`, replacing any handoff still waiting there.
    ///
    /// 422 when no such item exists — there would be nothing to hand off.
    pub async fn create_for_store(
        &self,
        store: Store,
        delivery: DeliveryRequest,
        user_id: &str,
    ) -> Result<HandoffWithLines, ApiError> {
        delivery::check_requested_date(&delivery, Utc::now().date_naive())?;
        let mut tx = self.pool.begin().await?;
        let chosen = line_repository::chosen_lines_for_store(&mut *tx, store).await?;
        if chosen.is_empty() {
            tx.rollback().await?;
            log::nothing_to_hand_off(store);
            return Err(ApiError::UnprocessableEntity(format!(
                "No item on the list has a {} product chosen yet.",
                store.display_name()
            )));
        }
        let replaced = repository::replace_waiting(&mut *tx, store).await?;
        let expires_at = Utc::now() + Duration::minutes(HANDOFF_LIFETIME_MINUTES);
        let handoff =
            repository::insert_handoff(&mut *tx, store, user_id, expires_at, &delivery).await?;
        for (position, line) in chosen.iter().enumerate() {
            let position = i32::try_from(position).unwrap_or(i32::MAX);
            line_repository::insert_line(&mut *tx, handoff.id, position, line).await?;
        }
        let lines = line_repository::lines_of(&mut *tx, handoff.id).await?;
        tx.commit().await?;
        log::created(&handoff, lines.len(), replaced);
        Ok(HandoffWithLines { handoff, lines })
    }

    /// A handoff and its lines; 404 when there is no such handoff.
    pub async fn find(&self, id: Uuid) -> Result<HandoffWithLines, ApiError> {
        let handoff = repository::find_handoff(self.pool, id)
            .await?
            .ok_or_else(|| not_found(id))?;
        let lines = line_repository::lines_of(self.pool, id).await?;
        Ok(HandoffWithLines { handoff, lines })
    }

    /// Claims the newest unexpired waiting handoff for `store`, for the
    /// store tab to fill. `None` when nothing is waiting.
    pub async fn claim_for_store_tab(
        &self,
        store: Store,
    ) -> Result<Option<ClaimedHandoff>, ApiError> {
        let mut tx = self.pool.begin().await?;
        let Some(handoff) = repository::claim_newest_waiting(&mut *tx, store).await? else {
            tx.rollback().await?;
            log::nothing_to_claim(store);
            return Ok(None);
        };
        let lines =
            store_tab::lines_for_store_tab(&line_repository::lines_of(&mut *tx, handoff.id).await?);
        tx.commit().await?;
        log::claimed(&handoff, lines.len());
        Ok(Some(ClaimedHandoff { handoff, lines }))
    }

    /// Records the store tab's report and closes the handoff.
    ///
    /// 404 unknown handoff; 409 it is not claimed (never claimed, already
    /// reported, or replaced); 422 the report does not cover exactly its
    /// products.
    pub async fn record_store_tab_report(
        &self,
        id: Uuid,
        report: &[ReportedLine],
        delivery: Option<&ReportedDelivery>,
    ) -> Result<HandoffWithLines, ApiError> {
        let mut tx = self.pool.begin().await?;
        let handoff = repository::lock_handoff(&mut *tx, id)
            .await?
            .ok_or_else(|| not_found(id))?;
        if handoff.status != TrolleyHandoffStatus::ClaimedByStoreTab {
            tx.rollback().await?;
            log::report_refused(id, "not claimed");
            return Err(ApiError::Conflict(
                "This handoff is not waiting for a report.".to_string(),
            ));
        }
        let lines = line_repository::lines_of(&mut *tx, id).await?;
        store_tab::check_report(&lines, report)
            .and_then(|()| delivery.map_or(Ok(()), delivery::check_delivery_report))
            .inspect_err(|err| log::report_refused(id, &err.to_string()))?;
        for line in report {
            line_repository::set_outcome(
                &mut *tx,
                id,
                &line.product_id,
                line.outcome,
                line.problem.as_deref(),
            )
            .await?;
        }
        let handoff = repository::finish(
            &mut *tx,
            id,
            store_tab::status_after_report(report),
            delivery,
        )
        .await?;
        let lines = line_repository::lines_of(&mut *tx, id).await?;
        tx.commit().await?;
        log::reported(&handoff, &lines);
        Ok(HandoffWithLines { handoff, lines })
    }
}

/// The 404 for an unknown handoff id.
fn not_found(id: Uuid) -> ApiError {
    ApiError::NotFound(format!("Trolley handoff {id} not found"))
}
