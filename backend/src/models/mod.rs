//! Database row types and the API's request/response shapes.
//!
//! `db` mirrors the tables; `schemas` mirrors the JSON the web app and the
//! intake channels exchange. Neither module contains business logic.

pub mod db;
pub mod google_tasks_rows;
pub mod schemas;
pub mod trolley_handoff_rows;
