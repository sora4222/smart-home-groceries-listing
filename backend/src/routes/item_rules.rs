//! Item rules: the `/settings/item-rules` page's list, add, edit and delete.
//!
//! Every route needs a signed-in household member. Applying a rule to an item
//! is not a route — it happens as items reach the list, in the grocery and
//! intake services.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{ItemRuleCreate, ItemRuleResponse, ItemRuleUpdate};
use crate::routes::extract::ValidatedJson;
use crate::services::item_rules::ItemRuleService;
use crate::state::AppState;

/// Routes under `/api/item-rules`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/item-rules", get(list_rules).post(create_rule))
        .route(
            "/api/item-rules/{rule_id}",
            patch(update_rule).delete(delete_rule),
        )
}

/// `GET /api/item-rules` — every rule, in the order they were made.
async fn list_rules(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<ItemRuleResponse>>, ApiError> {
    let rules = ItemRuleService::new(&state.pool).list().await?;
    Ok(Json(rules.into_iter().map(Into::into).collect()))
}

/// `POST /api/item-rules` — add a rule. 201 with the stored rule.
async fn create_rule(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedJson(body): ValidatedJson<ItemRuleCreate>,
) -> Result<(StatusCode, Json<ItemRuleResponse>), ApiError> {
    let rule = ItemRuleService::new(&state.pool).create(&body).await?;
    Ok((StatusCode::CREATED, Json(rule.into())))
}

/// `PATCH /api/item-rules/{id}` — change triggers, terms or the manual toggle.
async fn update_rule(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(rule_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ItemRuleUpdate>,
) -> Result<Json<ItemRuleResponse>, ApiError> {
    let rule = ItemRuleService::new(&state.pool)
        .update(rule_id, &body)
        .await?;
    Ok(Json(rule.into()))
}

/// `DELETE /api/item-rules/{id}` — remove a rule. 204.
async fn delete_rule(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    ItemRuleService::new(&state.pool).delete(rule_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
