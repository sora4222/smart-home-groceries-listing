//! App-wide request middleware written for this backend. Off-the-shelf layers
//! (CORS, timeouts, body limits, tracing) are wired in `lib.rs` directly.

pub mod access_log;
