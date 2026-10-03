//! Google Tasks intake: connecting the account, choosing the list, and
//! polling it now. Every route needs a signed-in household member.
//!
//! The sign-in round trip comes back to the web app's `/settings/intake`
//! page (the OAuth redirect), which posts the code and state here with the
//! member's own session — so no route here is open to the internet.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    FakeTaskCreate, GoogleSignInFinish, GoogleSignInStart, GoogleTasksChoices, GoogleTasksStatus,
    PollReport, TaskListResponse,
};
use crate::routes::extract::ValidatedJson;
use crate::services::google_tasks::fake::FAKE_GROCERIES_LIST;
use crate::services::google_tasks::{TasksConnection, TasksPoller};
use crate::state::AppState;

/// Routes under `/api/intake/google-tasks`, plus the fake's test door.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/intake/google-tasks", put(save_choices))
        .route(
            "/api/intake/google-tasks/sign-in",
            post(start_sign_in).delete(disconnect),
        )
        .route(
            "/api/intake/google-tasks/sign-in/finish",
            post(finish_sign_in),
        )
        .route("/api/intake/google-tasks/lists", get(list_task_lists))
        .route("/api/intake/google-tasks/poll", post(poll_now))
        .route("/api/dev/google-tasks/tasks", post(add_fake_task))
}

fn connection(state: &AppState) -> TasksConnection<'_> {
    TasksConnection::new(&state.pool, &state.google_tasks, state.encryptor.as_ref())
}

/// The channel's status after a change, as the settings page shows it.
pub fn status_of(
    state: &AppState,
    link: Option<&crate::models::db::GoogleTasksLink>,
) -> GoogleTasksStatus {
    GoogleTasksStatus::from_link(
        link,
        state.google_tasks.configured,
        state.encryptor.is_some(),
    )
}

/// `POST /api/intake/google-tasks/sign-in` — where to send the browser.
async fn start_sign_in(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<GoogleSignInStart>, ApiError> {
    let authorize_url = connection(&state).start_sign_in().await?;
    Ok(Json(GoogleSignInStart { authorize_url }))
}

/// `POST /api/intake/google-tasks/sign-in/finish` — store what Google sent
/// back. 409 for an unknown, expired or reused state.
async fn finish_sign_in(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<GoogleSignInFinish>,
) -> Result<Json<GoogleTasksStatus>, ApiError> {
    let link = connection(&state)
        .finish_sign_in(&body.code, &body.state, &user.id)
        .await?;
    Ok(Json(status_of(&state, Some(&link))))
}

/// `DELETE /api/intake/google-tasks/sign-in` — forget the sign-in.
async fn disconnect(State(state): State<AppState>, user: AuthUser) -> Result<StatusCode, ApiError> {
    connection(&state).disconnect(&user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/intake/google-tasks/lists` — the lists on the account.
async fn list_task_lists(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<TaskListResponse>>, ApiError> {
    let lists = connection(&state).task_lists().await?;
    Ok(Json(
        lists
            .into_iter()
            .map(|list| TaskListResponse {
                id: list.id,
                title: list.title,
            })
            .collect(),
    ))
}

/// `PUT /api/intake/google-tasks` — the list, the switch and the interval.
async fn save_choices(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<GoogleTasksChoices>,
) -> Result<Json<GoogleTasksStatus>, ApiError> {
    let link = connection(&state).save_choices(&body, &user.id).await?;
    Ok(Json(status_of(&state, Some(&link))))
}

/// `POST /api/intake/google-tasks/poll` — "Check now". 409 when polling is
/// not set up yet.
async fn poll_now(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PollReport>, ApiError> {
    TasksPoller::new(
        &state.pool,
        &state.hub,
        &state.triage,
        &state.google_tasks,
        state.encryptor.as_ref(),
    )
    .poll_once()
    .await?
    .map(Json)
    .ok_or_else(|| {
        ApiError::Conflict("Connect Google Tasks, pick a list and switch it on first".into())
    })
}

/// `POST /api/dev/google-tasks/tasks` — add a task to the **fake** account's
/// "Groceries" list, the way a person would in the Tasks app. 404 unless
/// `GOOGLE_TASKS_CLIENT=fake`, so it does not exist against the real Google.
async fn add_fake_task(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedJson(body): ValidatedJson<FakeTaskCreate>,
) -> Result<StatusCode, ApiError> {
    let fake = state
        .google_tasks
        .fake
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;
    fake.add(FAKE_GROCERIES_LIST, &body.title)?;
    Ok(StatusCode::CREATED)
}
