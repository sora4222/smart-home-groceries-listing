//! The real Google Tasks client: the Tasks REST API v1 over `reqwest`.
//!
//! Google is not a store, so plain `reqwest` is right here (the `wreq` rule
//! is about the stores' bot protection). A failed call is reported, never
//! retried in a loop; the next poll is the retry.

mod oauth;

use reqwest::{Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use self::oauth::OAuth;
use super::api::{GoogleError, RemoteTask, TaskList, TasksApi};
use crate::config::GoogleTasksSettings;
use crate::services::stores::client::BoxFuture;

pub use oauth::{authorize_url, TASKS_SCOPE};

/// Most pages read from one listing. 100 items a page, so 1000 tasks.
const MAX_PAGES: usize = 10;

/// One page of a Tasks API listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
    #[serde(default = "Vec::new")]
    items: Vec<T>,
    next_page_token: Option<String>,
}

/// A task as the API describes it. Only the fields used are read.
#[derive(Debug, Deserialize)]
struct ApiTask {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    status: String,
}

/// A list as the API describes it.
#[derive(Debug, Deserialize)]
struct ApiList {
    id: String,
    #[serde(default)]
    title: String,
}

/// Google Tasks over HTTPS.
pub struct LiveTasksApi {
    http: reqwest::Client,
    google_sign_in: OAuth,
    api_base: String,
}

impl LiveTasksApi {
    /// A client for the household's Google Cloud project.
    pub fn new(http: reqwest::Client, settings: &GoogleTasksSettings) -> Self {
        Self {
            google_sign_in: OAuth::new(http.clone(), settings),
            http,
            api_base: settings.api_base.trim_end_matches('/').to_string(),
        }
    }

    /// Sends one authorised call. A 401 means the access was taken away, so
    /// the cached token is dropped and the household is told to reconnect.
    async fn send(
        &self,
        refresh_token: &str,
        build: impl FnOnce(&reqwest::Client) -> RequestBuilder,
    ) -> Result<reqwest::Response, GoogleError> {
        let token = self.google_sign_in.access_token(refresh_token).await?;
        let response = build(&self.http)
            .bearer_auth(token)
            .send()
            .await
            .map_err(unreachable)?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED => {
                self.google_sign_in.forget().await;
                Err(GoogleError::SignInRevoked)
            }
            StatusCode::NOT_FOUND => Err(GoogleError::NotFound),
            status => Err(status_error(status)),
        }
    }

    /// Reads every page of a listing at `url` with `query`.
    async fn list_all<T: DeserializeOwned>(
        &self,
        refresh_token: &str,
        url: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<T>, GoogleError> {
        let mut items = Vec::new();
        let mut page_token: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut params: Vec<(&str, &str)> = query.to_vec();
            params.push(("maxResults", "100"));
            if let Some(token) = page_token.as_deref() {
                params.push(("pageToken", token));
            }
            let response = self
                .send(refresh_token, |http| http.get(url).query(&params))
                .await?;
            let page: Page<T> = response.json().await.map_err(|_| GoogleError::BadReply)?;
            items.extend(page.items);
            match page.next_page_token {
                Some(next) if !next.is_empty() => page_token = Some(next),
                _ => break,
            }
        }
        Ok(items)
    }

    fn tasks_url(&self, list_id: &str) -> String {
        format!(
            "{}/tasks/v1/lists/{}/tasks",
            self.api_base,
            segment(list_id)
        )
    }
}

impl TasksApi for LiveTasksApi {
    fn authorize_url(&self, state: &str) -> String {
        self.google_sign_in.authorize_url(state)
    }

    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, GoogleError>> {
        Box::pin(self.google_sign_in.exchange_code(code))
    }

    fn task_lists<'a>(
        &'a self,
        refresh_token: &'a str,
    ) -> BoxFuture<'a, Result<Vec<TaskList>, GoogleError>> {
        Box::pin(async move {
            let url = format!("{}/tasks/v1/users/@me/lists", self.api_base);
            let lists: Vec<ApiList> = self.list_all(refresh_token, &url, &[]).await?;
            Ok(lists
                .into_iter()
                .map(|list| TaskList {
                    id: list.id,
                    title: list.title,
                })
                .collect())
        })
    }

    fn open_tasks<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<RemoteTask>, GoogleError>> {
        Box::pin(async move {
            let url = self.tasks_url(list_id);
            let query = [("showCompleted", "false"), ("showHidden", "false")];
            let tasks: Vec<ApiTask> = self.list_all(refresh_token, &url, &query).await?;
            Ok(tasks
                .into_iter()
                .filter(|task| task.status != "completed")
                .map(|task| RemoteTask {
                    id: task.id,
                    title: task.title,
                })
                .collect())
        })
    }

    fn delete_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        task_id: &'a str,
    ) -> BoxFuture<'a, Result<(), GoogleError>> {
        Box::pin(async move {
            let url = format!("{}/{}", self.tasks_url(list_id), segment(task_id));
            match self
                .send(refresh_token, |http| http.request(Method::DELETE, &url))
                .await
            {
                Ok(_) | Err(GoogleError::NotFound) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    fn insert_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        title: &'a str,
    ) -> BoxFuture<'a, Result<RemoteTask, GoogleError>> {
        Box::pin(async move {
            let url = self.tasks_url(list_id);
            let body = serde_json::json!({ "title": title });
            let response = self
                .send(refresh_token, |http| http.post(&url).json(&body))
                .await?;
            let task: ApiTask = response.json().await.map_err(|_| GoogleError::BadReply)?;
            Ok(RemoteTask {
                id: task.id,
                title: task.title,
            })
        })
    }
}

/// Percent-encodes one path segment, so an id can never add a path.
fn segment(raw: &str) -> String {
    url::form_urlencoded::byte_serialize(raw.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

/// A transport failure, described without the URL or any header.
fn unreachable(err: reqwest::Error) -> GoogleError {
    GoogleError::Unavailable(if err.is_timeout() {
        "timed out".to_string()
    } else {
        "no connection".to_string()
    })
}

/// A refusal, described by its status alone.
fn status_error(status: StatusCode) -> GoogleError {
    GoogleError::Unavailable(format!("answered {}", status.as_u16()))
}

#[cfg(test)]
mod tests {
    use super::segment;

    #[test]
    fn an_id_cannot_climb_out_of_its_path_segment() {
        assert_eq!(segment("abc"), "abc");
        assert_eq!(segment("../x"), "..%2Fx");
        assert_eq!(segment("a b"), "a%20b");
    }
}
