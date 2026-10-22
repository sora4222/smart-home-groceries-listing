//! The one trolley fill running at each store, from the web app's request to
//! the result the web app receives.
//!
//! `fill_store_trolley` begins an operation and holds its [`FillTicket`]. The
//! store window starts the program when the trolley page loads
//! (`waiting` → `running`). The watcher (`fill_watch.rs`) or a closed window
//! finishes it with a [`FillOutcome`]. A second fill at the same store is
//! refused until the first ends, so it cannot navigate the window while the
//! first is changing quantities. Dropping the ticket (timeout, failed window,
//! cancelled command) ends the operation, so no program is left behind.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;
use tokio::sync::oneshot;
use url::Url;

use crate::fill_failure::FillFailure;
use crate::stores::Store;

/// What the web app receives when a fill completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FillReport {
    /// The fill program's own sentence, e.g. "Added 5 products."
    pub message: String,
}

/// How an operation ended: the report, or why it failed.
pub type FillOutcome = Result<FillReport, FillFailure>;

/// Where an operation is.
enum Phase {
    /// The store window is on its way to the trolley page (or a login page).
    Waiting(String),
    /// The program was started on the trolley page.
    Running,
}

struct Operation {
    id: u64,
    phase: Phase,
    done: oneshot::Sender<FillOutcome>,
}

#[derive(Default)]
struct State {
    next_id: u64,
    operations: HashMap<Store, Operation>,
}

/// Every store's current operation.
#[derive(Default)]
pub struct FillOperations(Mutex<State>);

/// The caller's hold on one operation. Await [`FillTicket::outcome`]; dropping
/// the ticket first abandons the operation.
pub struct FillTicket<'a> {
    operations: &'a FillOperations,
    store: Store,
    id: u64,
    done: Option<oneshot::Receiver<FillOutcome>>,
}

impl FillOperations {
    /// Begins a fill of `store` with `program`, unless one is already going.
    pub fn begin(&self, store: Store, program: String) -> Result<FillTicket<'_>, FillFailure> {
        let mut state = self.lock();
        if state.operations.contains_key(&store) {
            return Err(FillFailure::Busy(store));
        }
        state.next_id += 1;
        let id = state.next_id;
        let (done, receiver) = oneshot::channel();
        let phase = Phase::Waiting(program);
        state
            .operations
            .insert(store, Operation { id, phase, done });
        Ok(FillTicket {
            operations: self,
            store,
            id,
            done: Some(receiver),
        })
    }

    /// The program to start, if `page` is `store`'s trolley page and an
    /// operation is waiting for it. The operation is then running, so the
    /// program starts once.
    pub fn start_on(&self, store: Store, page: &Url) -> Option<String> {
        if !store.is_trolley_page(page) {
            return None;
        }
        let mut state = self.lock();
        let operation = state.operations.get_mut(&store)?;
        match std::mem::replace(&mut operation.phase, Phase::Running) {
            Phase::Waiting(program) => Some(program),
            Phase::Running => None,
        }
    }

    /// True while `store`'s program is running.
    pub fn is_running(&self, store: Store) -> bool {
        matches!(
            self.lock().operations.get(&store),
            Some(Operation {
                phase: Phase::Running,
                ..
            })
        )
    }

    /// Ends `store`'s current operation with `outcome`.
    pub fn finish(&self, store: Store, outcome: FillOutcome) {
        let operation = self.lock().operations.remove(&store);
        if let Some(operation) = operation {
            log::info!("[fill] {store:?} finished: {:?}", outcome.as_ref().err());
            // The caller may have gone already; the operation still ends.
            let _ = operation.done.send(outcome);
        }
    }

    /// Ends operation `id`, if it is still `store`'s current one.
    fn abandon(&self, store: Store, id: u64) {
        let mut state = self.lock();
        if state.operations.get(&store).is_some_and(|op| op.id == id) {
            state.operations.remove(&store);
            log::info!("[fill] {store:?} abandoned");
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // A panic while holding the lock leaves plain data behind; keep going.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl FillTicket<'_> {
    /// Waits for the operation to end.
    pub async fn outcome(mut self) -> FillOutcome {
        let done = self.done.take().expect("outcome is awaited once");
        done.await.unwrap_or(Err(FillFailure::Stopped))
    }
}

impl Drop for FillTicket<'_> {
    fn drop(&mut self) {
        self.operations.abandon(self.store, self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    const TROLLEY: &str = "https://www.woolworths.com.au/shop/mytrolley";

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tauri::async_runtime::block_on(future)
    }

    #[test]
    fn starts_the_program_once_on_the_trolley_page() {
        let operations = FillOperations::default();
        let _ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        assert!(!operations.is_running(Store::Woolworths));
        assert_eq!(
            operations.start_on(Store::Woolworths, &url(TROLLEY)),
            Some("fill()".into())
        );
        assert!(operations.is_running(Store::Woolworths));
        assert_eq!(operations.start_on(Store::Woolworths, &url(TROLLEY)), None);
    }

    #[test]
    fn a_login_page_does_not_start_it() {
        let operations = FillOperations::default();
        let _ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        let login = url("https://www.woolworths.com.au/shop/securelogin");
        assert_eq!(operations.start_on(Store::Woolworths, &login), None);
        let back = url("https://www.woolworths.com.au/shop/mytrolley/?from=login");
        assert_eq!(
            operations.start_on(Store::Woolworths, &back),
            Some("fill()".into())
        );
    }

    #[test]
    fn never_starts_on_another_website() {
        let operations = FillOperations::default();
        let _ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        let elsewhere = url("https://evil.example/shop/mytrolley");
        assert_eq!(operations.start_on(Store::Woolworths, &elsewhere), None);
        assert_eq!(operations.start_on(Store::Coles, &url(TROLLEY)), None);
    }

    #[test]
    fn refuses_a_second_fill_while_one_is_going() {
        let operations = FillOperations::default();
        let _first = operations.begin(Store::Woolworths, "one()".into()).unwrap();
        let refused = operations.begin(Store::Woolworths, "two()".into());
        assert_eq!(refused.err(), Some(FillFailure::Busy(Store::Woolworths)));
        assert!(operations.begin(Store::Coles, "other()".into()).is_ok());
    }

    #[test]
    fn hands_the_outcome_to_the_ticket_and_frees_the_store() {
        let operations = FillOperations::default();
        let ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        let report = FillReport {
            message: "Added 5 products.".into(),
        };
        operations.finish(Store::Woolworths, Ok(report.clone()));
        assert_eq!(block_on(ticket.outcome()), Ok(report));
        assert!(operations.begin(Store::Woolworths, "next()".into()).is_ok());
    }

    #[test]
    fn a_vanished_operation_is_stopped() {
        let operations = FillOperations::default();
        let ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        operations.lock().operations.clear();
        assert_eq!(block_on(ticket.outcome()), Err(FillFailure::Stopped));
    }

    #[test]
    fn dropping_the_ticket_abandons_the_waiting_program() {
        let operations = FillOperations::default();
        let ticket = operations
            .begin(Store::Woolworths, "fill()".into())
            .unwrap();
        drop(ticket);
        assert_eq!(operations.start_on(Store::Woolworths, &url(TROLLEY)), None);
        assert!(operations.begin(Store::Woolworths, "next()".into()).is_ok());
    }

    #[test]
    fn an_old_ticket_never_abandons_a_newer_operation() {
        let operations = FillOperations::default();
        let old = operations.begin(Store::Woolworths, "old()".into()).unwrap();
        operations.finish(Store::Woolworths, Err(FillFailure::PageChanged));
        let _new = operations.begin(Store::Woolworths, "new()".into()).unwrap();
        drop(old);
        assert_eq!(
            operations.start_on(Store::Woolworths, &url(TROLLEY)),
            Some("new()".into())
        );
    }
}
