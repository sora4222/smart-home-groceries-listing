//! Request-body extractors that validate before a handler sees the data.
//!
//! These replace Pydantic's model-level constraints: a body that violates a
//! length or range limit is answered with 422 and never reaches a service,
//! so a service can assume its inputs are within bounds.

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ApiError;

/// A required JSON body, rejected with 422 unless it validates.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| ApiError::UnprocessableEntity(rejection.body_text()))?;
        value
            .validate()
            .map_err(|errors| ApiError::UnprocessableEntity(errors.to_string()))?;
        Ok(Self(value))
    }
}

/// An optional JSON body: an absent or empty body becomes `T::default()`.
///
/// The accept endpoint carries corrections the user may not have made, and
/// `POST` with no body is how both the web app's "accept as heard" path and a
/// plain `curl` call behave.
pub struct OptionalValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for OptionalValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Default + 'static,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::Bytes::from_request(request, state)
            .await
            .map_err(|rejection| ApiError::UnprocessableEntity(rejection.body_text()))?;

        // Treat an empty body and the empty object alike, so `POST` with no
        // body means "no corrections" rather than a malformed request.
        if bytes.iter().all(u8::is_ascii_whitespace) {
            return Ok(Self(T::default()));
        }

        let value: T = serde_json::from_slice(&bytes)
            .map_err(|err| ApiError::UnprocessableEntity(err.to_string()))?;
        value
            .validate()
            .map_err(|errors| ApiError::UnprocessableEntity(errors.to_string()))?;
        Ok(Self(value))
    }
}

/// A JSON body read whatever its `Content-Type`, rejected with 422 unless it
/// validates.
///
/// For the store tab (the bookmarklet on the store's website): it sends JSON
/// as `text/plain` so the browser treats the request as a CORS "simple
/// request" and sends no preflight, which the app-wide CORS layer would
/// otherwise answer for the web app's origin only.
pub struct AnyContentTypeJson<T>(pub T);

impl<S, T> FromRequest<S> for AnyContentTypeJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::Bytes::from_request(request, state)
            .await
            .map_err(|rejection| ApiError::UnprocessableEntity(rejection.body_text()))?;
        let value: T = serde_json::from_slice(&bytes)
            .map_err(|err| ApiError::UnprocessableEntity(err.to_string()))?;
        value
            .validate()
            .map_err(|errors| ApiError::UnprocessableEntity(errors.to_string()))?;
        Ok(Self(value))
    }
}

/// A query string, rejected with 422 unless it parses and validates.
pub struct ValidatedQuery<T>(pub T);

impl<S, T> axum::extract::FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) =
            axum::extract::Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|rejection| ApiError::UnprocessableEntity(rejection.body_text()))?;
        value
            .validate()
            .map_err(|errors| ApiError::UnprocessableEntity(errors.to_string()))?;
        Ok(Self(value))
    }
}
