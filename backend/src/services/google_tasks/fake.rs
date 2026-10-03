//! An in-memory Google Tasks account, for development, tests and e2e
//! (`GOOGLE_TASKS_CLIENT=fake`). Nothing leaves the machine.
//!
//! It starts with two lists, "Groceries" and "My Tasks". Signing in skips
//! Google entirely: the sign-in URL points straight back at the redirect
//! page with a code the fake accepts.

use std::sync::Mutex;

use super::api::{GoogleError, RemoteTask, TaskList, TasksApi};
use crate::services::stores::client::BoxFuture;

/// The code the fake sign-in hands back, and the token it trades it for.
pub const FAKE_CODE: &str = "fake-code";
const FAKE_REFRESH_TOKEN: &str = "fake-refresh-token";

/// The fake's "Groceries" list id.
pub const FAKE_GROCERIES_LIST: &str = "fake-groceries";

/// One fake list and its open tasks.
struct FakeList {
    list: TaskList,
    tasks: Vec<RemoteTask>,
}

/// The fake account.
pub struct FakeTasksApi {
    redirect_uri: String,
    lists: Mutex<Vec<FakeList>>,
    next_id: Mutex<u64>,
}

impl FakeTasksApi {
    /// A fresh account; sign-in returns to `redirect_uri`.
    pub fn new(redirect_uri: &str) -> Self {
        let list = |id: &str, title: &str| FakeList {
            list: TaskList {
                id: id.to_string(),
                title: title.to_string(),
            },
            tasks: Vec::new(),
        };
        Self {
            redirect_uri: redirect_uri.to_string(),
            lists: Mutex::new(vec![
                list(FAKE_GROCERIES_LIST, "Groceries"),
                list("fake-my-tasks", "My Tasks"),
            ]),
            next_id: Mutex::new(1),
        }
    }

    /// Adds a task the way a person would in the Tasks app. Used by the
    /// development-only route and the tests.
    pub fn add(&self, list_id: &str, title: &str) -> Result<RemoteTask, GoogleError> {
        let task = RemoteTask {
            id: self.new_id(),
            title: title.to_string(),
        };
        let mut lists = self.lists.lock().expect("fake lists lock");
        let list = lists
            .iter_mut()
            .find(|l| l.list.id == list_id)
            .ok_or(GoogleError::NotFound)?;
        list.tasks.push(task.clone());
        Ok(task)
    }

    /// The open tasks on a list, for assertions.
    pub fn tasks(&self, list_id: &str) -> Vec<RemoteTask> {
        let lists = self.lists.lock().expect("fake lists lock");
        lists
            .iter()
            .find(|l| l.list.id == list_id)
            .map(|l| l.tasks.clone())
            .unwrap_or_default()
    }

    fn new_id(&self) -> String {
        let mut next = self.next_id.lock().expect("fake id lock");
        *next += 1;
        format!("fake-task-{}", *next)
    }

    fn check(refresh_token: &str) -> Result<(), GoogleError> {
        if refresh_token == FAKE_REFRESH_TOKEN {
            Ok(())
        } else {
            Err(GoogleError::SignInRevoked)
        }
    }
}

impl TasksApi for FakeTasksApi {
    fn authorize_url(&self, state: &str) -> String {
        let mut url = match url::Url::parse(&self.redirect_uri) {
            Ok(url) => url,
            Err(_) => return self.redirect_uri.clone(),
        };
        url.query_pairs_mut()
            .append_pair("code", FAKE_CODE)
            .append_pair("state", state);
        url.into()
    }

    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, GoogleError>> {
        Box::pin(async move {
            if code == FAKE_CODE {
                Ok(FAKE_REFRESH_TOKEN.to_string())
            } else {
                Err(GoogleError::SignInRevoked)
            }
        })
    }

    fn task_lists<'a>(
        &'a self,
        refresh_token: &'a str,
    ) -> BoxFuture<'a, Result<Vec<TaskList>, GoogleError>> {
        Box::pin(async move {
            Self::check(refresh_token)?;
            let lists = self.lists.lock().expect("fake lists lock");
            Ok(lists.iter().map(|l| l.list.clone()).collect())
        })
    }

    fn open_tasks<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<RemoteTask>, GoogleError>> {
        Box::pin(async move {
            Self::check(refresh_token)?;
            let lists = self.lists.lock().expect("fake lists lock");
            lists
                .iter()
                .find(|l| l.list.id == list_id)
                .map(|l| l.tasks.clone())
                .ok_or(GoogleError::NotFound)
        })
    }

    fn delete_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        task_id: &'a str,
    ) -> BoxFuture<'a, Result<(), GoogleError>> {
        Box::pin(async move {
            Self::check(refresh_token)?;
            let mut lists = self.lists.lock().expect("fake lists lock");
            if let Some(list) = lists.iter_mut().find(|l| l.list.id == list_id) {
                list.tasks.retain(|t| t.id != task_id);
            }
            Ok(())
        })
    }

    fn insert_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        title: &'a str,
    ) -> BoxFuture<'a, Result<RemoteTask, GoogleError>> {
        Box::pin(async move {
            Self::check(refresh_token)?;
            self.add(list_id, title)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn signing_in_goes_straight_back_with_a_code_it_accepts() {
        let fake = FakeTasksApi::new("http://localhost:3000/settings/intake");
        let url = fake.authorize_url("s1");
        assert_eq!(
            url,
            "http://localhost:3000/settings/intake?code=fake-code&state=s1"
        );
        let token = fake.exchange_code(FAKE_CODE).await.unwrap();
        assert_eq!(fake.task_lists(&token).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn added_tasks_are_listed_until_deleted() {
        let fake = FakeTasksApi::new("http://x/");
        let task = fake.add(FAKE_GROCERIES_LIST, "milk").unwrap();
        let token = FAKE_REFRESH_TOKEN;
        assert_eq!(
            fake.open_tasks(token, FAKE_GROCERIES_LIST).await.unwrap(),
            vec![task.clone()]
        );
        fake.delete_task(token, FAKE_GROCERIES_LIST, &task.id)
            .await
            .unwrap();
        assert!(fake.tasks(FAKE_GROCERIES_LIST).is_empty());
    }

    #[tokio::test]
    async fn a_wrong_token_is_refused() {
        let fake = FakeTasksApi::new("http://x/");
        assert_eq!(
            fake.task_lists("nope").await.unwrap_err(),
            GoogleError::SignInRevoked
        );
    }
}
