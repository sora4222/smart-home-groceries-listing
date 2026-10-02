//! Google Tasks intake: the household's grocery list in Google Tasks is
//! polled, each task becomes an intake request, and the task is deleted once
//! the request is stored. See `docs/features/FEATURE_GOOGLE_TASKS.md`.
//!
//! | File | Job |
//! |---|---|
//! | `api.rs` | the [`TasksApi`] trait, its types and errors |
//! | `live/` | the real Google API (OAuth + Tasks REST) over `reqwest` |
//! | `fake.rs` | an in-memory account for development, tests and e2e |
//! | `registry.rs` | picks the client from `GOOGLE_TASKS_CLIENT` |
//! | `title.rs` | "2 oat milk" → name and quantity (pure) |
//! | `sign_in.rs` | sealing and opening the stored refresh token |
//! | `oauth_states.rs` | one-use sign-in `state` values (SQL) |
//! | `link_repository.rs` | the stored link and choices (SQL) |
//! | `connection.rs` | connect, disconnect, lists, choices |
//! | `poller.rs` | one poll: record, then delete |
//! | `schedule.rs` | the background poll loop |
//! | `restore.rs` | Triage's "Restore to source" |
//! | `log.rs` | log events |

pub mod api;
mod connection;
pub mod fake;
pub mod link_repository;
pub mod live;
mod log;
mod oauth_states;
mod poller;
pub mod registry;
mod restore;
pub mod schedule;
mod sign_in;
pub mod title;

pub use api::{GoogleError, RemoteTask, TaskList, TasksApi};
pub use connection::TasksConnection;
pub use fake::FakeTasksApi;
pub use poller::TasksPoller;
pub use registry::GoogleTasks;
pub use restore::TasksRestore;
