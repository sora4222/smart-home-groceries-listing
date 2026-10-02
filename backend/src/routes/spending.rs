//! The spending analysis behind the Analysis page.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use chrono_tz::Tz;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    ItemPriceResponse, ItemPricesParams, SpendingParams, SpendingResponse,
};
use crate::routes::extract::ValidatedQuery;
use crate::services::spending::{SpendingQuery, SpendingService};
use crate::state::AppState;

/// `/api/spending` and `/api/spending/item-prices`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/spending", get(spending))
        .route("/api/spending/item-prices", get(item_prices))
}

/// `GET /api/spending` — every view (over time, by item, by store, by
/// category) of the purchases the filters let through, plus the newest shop.
/// 422 for an unknown time zone or store, or `from` after `to`.
async fn spending(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedQuery(params): ValidatedQuery<SpendingParams>,
) -> Result<Json<SpendingResponse>, ApiError> {
    let query = to_query(params)?;
    let report = SpendingService::new(&state.pool).analyse(&query).await?;
    Ok(Json(report.into()))
}

/// `GET /api/spending/item-prices?name=` — what one item cost each time it
/// was bought, at any store, oldest first.
async fn item_prices(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedQuery(params): ValidatedQuery<ItemPricesParams>,
) -> Result<Json<Vec<ItemPriceResponse>>, ApiError> {
    let prices = SpendingService::new(&state.pool)
        .item_prices(&params.name)
        .await?;
    Ok(Json(prices.into_iter().map(Into::into).collect()))
}

/// Checks the query and turns blank filters into no filter.
fn to_query(params: SpendingParams) -> Result<SpendingQuery, ApiError> {
    if let (Some(from), Some(to)) = (params.from, params.to) {
        if from > to {
            return Err(ApiError::UnprocessableEntity(
                "The start day is after the end day.".to_string(),
            ));
        }
    }
    let tz = match params
        .tz
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => Tz::UTC,
        Some(name) => name
            .parse::<Tz>()
            .map_err(|_| ApiError::UnprocessableEntity(format!("Unknown time zone: {name}")))?,
    };
    Ok(SpendingQuery {
        from: params.from,
        to: params.to,
        store: params.store,
        item: not_blank(params.item),
        category: not_blank(params.category),
        period: params.period,
        tz,
    })
}

/// `None` for an absent or blank filter.
fn not_blank(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}
